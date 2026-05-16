//! HTTP request handlers

use crate::error::ApiError;
use crate::middleware::CallerIdentity;
use crate::state::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
};
use qorvum_ledger::{
    block::*,
    query::{Filter, Pagination, SortBy},
    record::FieldValue,
    store::RecordOp,
};
use qorvum_msp::{CertSubject, QorvumToken};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

// ── Shared response wrapper ───────────────────────────────────────────────────

fn ok(data: Value) -> Json<Value> {
    Json(json!({ "success": true, "data": data }))
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn make_tx_id(caller: &str, ts: u64) -> [u8; 32] {
    qorvum_crypto::hash_many(&[caller.as_bytes(), &ts.to_le_bytes()])
}

// ── POST /api/v1/invoke/:contract/:function ───────────────────────────────────

pub async fn invoke(
    State(state):   State<Arc<AppState>>,
    Path((contract, function)): Path<(String, String)>,
    Extension(caller): Extension<CallerIdentity>,
    Json(args):     Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let ts    = now_nanos();
    let tx_id = make_tx_id(&caller.id, ts);

    let result = {
        let executor = state.executor.read().await;
        executor.execute(
            &contract, &function, args.clone(),
            &caller.id, &caller.org, caller.roles.clone(),
            tx_id, ts,
            caller.verified,
        ).map_err(ApiError::BadRequest)?
    };

    if result.ops.is_empty() {
        return Ok(ok(json!({ "tx_id": hex::encode(tx_id), "response": result.response })));
    }

    let block_num = crate::commit::commit_block_with_events(
        &state, tx_id, ts, &contract, &function, args, result.ops, &caller.id
    ).await?;

    Ok(ok(json!({
        "tx_id":     hex::encode(tx_id),
        "block_num": block_num,
        "response":  result.response,
    })))
}

/// Build a block and commit it — either via the ConsensusEngine (multi-node)
/// or directly to the store (single-node dev mode).
async fn commit_block(
    state:    &AppState,
    tx_id:    [u8; 32],
    ts:       u64,
    contract: &str,
    function: &str,
    args:     Value,
    ops:      Vec<RecordOp>,
) -> Result<u64, ApiError> {
    let (block, block_num) = build_block(state, tx_id, ts, contract, function, args, &ops).await?;

    if let Some(ref engine) = state.consensus {
        // Multi-node path: run HotStuff consensus before committing
        let block_data = serde_json::to_vec(&(&block, &ops))
            .map_err(|e| ApiError::Internal(format!("serialize block_data: {e}")))?;

        engine.propose_block(block_data).await
            .map_err(|e| ApiError::Internal(format!("consensus: {e}")))?;
        // ConsensusEngine already committed the block to the store
    } else {
        // Single-node dev mode: commit directly
        state.store.commit_block(&block, ops)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    Ok(block_num)
}

/// Prepare a Block (and return its number) WITHOUT committing to the store.
async fn build_block(
    state:    &AppState,
    tx_id:    [u8; 32],
    ts:       u64,
    contract: &str,
    function: &str,
    args:     Value,
    _ops:     &[RecordOp],
) -> Result<(Block, u64), ApiError> {
    let latest = state.store.get_latest_block_num()
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .unwrap_or(0);
    let prev_hash = state.store.get_block(latest)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|b| b.compute_hash())
        .unwrap_or([0u8; 32]);

    let creator_sig = EndorsementSig { algorithm: "dev-mode".into(), bytes: vec![] };
    let tx = Transaction {
        tx_id,
        channel_id:    state.channel_id.clone(),
        contract_id:   contract.to_string(),
        function_name: function.to_string(),
        args,
        creator_pub_key: vec![],
        creator_sig:   creator_sig.clone(),
        endorsements:  vec![],
        nonce:         [0u8; 32],
        timestamp:     ts,
    };

    let mut builder = BlockBuilder::new(
        state.channel_id.clone(),
        latest + 1,
        prev_hash,
        "DefaultMSP".into(),
        vec![],
        creator_sig,
    );
    builder.add_transaction(tx);
    let block = builder.build();
    let block_num = block.header.block_number;

    Ok((block, block_num))
}

// ── GET /api/v1/query/:contract/:function ─────────────────────────────────────

#[derive(Deserialize)]
pub struct QueryParams {
    args: Option<String>,
}

pub async fn query(
    State(state):   State<Arc<AppState>>,
    Path((contract, function)): Path<(String, String)>,
    Extension(caller): Extension<CallerIdentity>,
    Query(params):  Query<QueryParams>,
) -> Result<Json<Value>, ApiError> {
    let args: Value = params.args
        .as_deref()
        .map(|s| serde_json::from_str(s).unwrap_or(Value::Null))
        .unwrap_or(Value::Null);

    let ts    = now_nanos();
    let tx_id = make_tx_id(&caller.id, ts);

    let executor = state.executor.read().await;
    let result = executor.execute(
        &contract, &function, args,
        &caller.id, &caller.org, caller.roles.clone(),
        tx_id, ts,
        caller.verified,
    ).map_err(ApiError::BadRequest)?;

    Ok(ok(result.response))
}

// ── GET /api/v1/records/:collection/:partition/:id ────────────────────────────

pub async fn get_record(
    State(state): State<Arc<AppState>>,
    Path((collection, partition, id)): Path<(String, String, String)>,
) -> Result<Json<Value>, ApiError> {
    let key = format!("{}~{}~{}", collection, partition, id);
    let record = state.store.get_record(&key)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("{}/{}/{}", collection, partition, id)))?;

    Ok(ok(serde_json::to_value(&record).unwrap()))
}

// ── GET /api/v1/records/:collection ──────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListParams {
    partition: Option<String>,
    limit:     Option<usize>,
    offset:    Option<usize>,
    sort:      Option<String>,
    status:    Option<String>,
    deleted:   Option<bool>,
}

pub async fn list_records(
    State(state):  State<Arc<AppState>>,
    Path(collection): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<Value>, ApiError> {
    let mut filters: Vec<Filter> = vec![];
    if params.deleted.unwrap_or(false) {
        filters.push(Filter::IncludeDeleted);
    }
    if let Some(s) = &params.status {
        filters.push(Filter::Eq("status".into(), FieldValue::Text(s.clone())));
    }
    let filter = match filters.len() {
        0 => None,
        1 => Some(filters.remove(0)),
        _ => Some(Filter::And(filters)),
    };
    let sort: Option<Vec<SortBy>> = params.sort.as_ref().map(|s| {
        s.split(',').map(|part| {
            let mut sp = part.splitn(2, ':');
            let field  = sp.next().unwrap_or("_id").to_string();
            let desc   = sp.next().unwrap_or("asc") == "desc";
            SortBy { field, descending: desc }
        }).collect()
    });
    let pag = Pagination { limit: params.limit.unwrap_or(20), offset: params.offset.unwrap_or(0) };

    let result = state.query_engine.query(
        &collection, params.partition.as_deref(),
        filter.as_ref(), sort.as_deref(), Some(&pag),
    ).map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ok(json!({ "records": result.records, "total": result.total, "offset": result.offset, "limit": result.limit })))
}

// ── GET /api/v1/history/:collection/:id ──────────────────────────────────────

pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let history = state.store.get_history(&collection, &id)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let entries: Vec<Value> = history.into_iter()
        .map(|(ver, block_num)| json!({ "version": ver, "block_num": block_num }))
        .collect();

    Ok(ok(json!({ "id": id, "collection": collection, "history": entries })))
}

// ── GET /api/v1/blocks/:number ────────────────────────────────────────────────

pub async fn get_block(
    State(state): State<Arc<AppState>>,
    Path(number): Path<u64>,
) -> Result<Json<Value>, ApiError> {
    let block = state.store.get_block(number)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Block {}", number)))?;

    Ok(ok(serde_json::to_value(&block).unwrap()))
}

// ── GET /api/v1/blocks?limit=&offset= ────────────────────────────────────────

#[derive(Deserialize)]
pub struct BlockListParams {
    limit:  Option<u64>,
    offset: Option<u64>,
}

pub async fn list_blocks(
    State(state):  State<Arc<AppState>>,
    Query(params): Query<BlockListParams>,
) -> Result<Json<Value>, ApiError> {
    let limit  = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let latest = state.store.get_latest_block_num()
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .unwrap_or(0);

    // newest first: start at (latest - offset), go back `limit` blocks
    let start = latest.saturating_sub(offset);
    let count = start.min(limit - 1); // how many blocks below `start` we fetch

    let mut blocks = Vec::new();
    for num in (start.saturating_sub(count)..=start).rev() {
        match state.store.get_block(num) {
            Ok(Some(block)) => blocks.push(serde_json::to_value(&block).unwrap_or(Value::Null)),
            Ok(None) => {}
            Err(e) => tracing::warn!("get_block({num}) failed: {e}"),
        }
    }

    Ok(ok(json!({
        "blocks": blocks,
        "total":  latest + 1,
        "offset": offset,
        "limit":  limit,
    })))
}

// ── GET /api/v1/stats ─────────────────────────────────────────────────────────

pub async fn get_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let latest = state.store.get_latest_block_num().unwrap_or(None);
    let mode   = if state.consensus.is_some() { "consensus" } else { "dev" };
    Json(json!({
        "success": true,
        "data": {
            "channel":      state.channel_id,
            "block_height": latest,
            "mode":         mode,
            "version":      env!("CARGO_PKG_VERSION"),
        }
    }))
}

// ── GET /api/v1/contracts ─────────────────────────────────────────────────────

pub async fn list_contracts(State(state): State<Arc<AppState>>) -> Json<Value> {
    let executor = state.executor.read().await;
    let contracts = executor.list_contracts();
    let total = contracts.len();
    Json(json!({ "success": true, "data": { "contracts": contracts, "total": total } }))
}

// ── GET /api/v1/health ────────────────────────────────────────────────────────

pub async fn health(State(state): State<Arc<AppState>>) -> Json<Value> {
    let latest  = state.store.get_latest_block_num().unwrap_or(None);
    let mode    = if state.consensus.is_some() { "consensus" } else { "dev" };
    Json(json!({
        "status":       "ok",
        "channel":      state.channel_id,
        "latest_block": latest,
        "mode":         mode,
        "version":      env!("CARGO_PKG_VERSION"),
    }))
}

// ── Helper: require ADMIN role ────────────────────────────────────────────────

fn require_admin(caller: &CallerIdentity) -> Result<(), ApiError> {
    if !caller.roles.iter().any(|r| r == "ADMIN") {
        return Err(ApiError::Forbidden("requires ADMIN role".into()));
    }
    Ok(())
}

// ── POST /api/v1/auth/login ───────────────────────────────────────────────────
// Public endpoint — no Bearer token needed.
// Decrypts the user's stored keypair and signs a fresh QorvumToken.

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// Token TTL in seconds (default: 3600)
    pub ttl: Option<u64>,
}

pub async fn auth_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_store = state.user_store.as_ref().ok_or_else(|| {
        ApiError::Internal(
            "Enrollment not configured — start the node with QORVUM_CA_PASSPHRASE".into(),
        )
    })?;

    let identity = user_store
        .load_identity(&body.username, &body.password)
        .map_err(|e| match e {
            qorvum_msp::MspError::UnknownUser(_) => {
                ApiError::Unauthorized("Invalid username or password".into())
            }
            qorvum_msp::MspError::WrongPassphrase => {
                ApiError::Unauthorized("Invalid username or password".into())
            }
            other => ApiError::Internal(other.to_string()),
        })?;

    let ttl = body.ttl.unwrap_or(3600);
    let token = QorvumToken::issue(&identity, ttl)
        .map_err(|e| ApiError::Internal(format!("Failed to issue token: {}", e)))?;

    let bearer = token
        .to_bearer_string()
        .map_err(|e| ApiError::Internal(format!("Failed to encode token: {}", e)))?;

    let expires_at = token.expires_at / 1_000_000_000;

    Ok(ok(json!({
        "token":      bearer,
        "expires_at": expires_at,
        "subject":    token.claims.subject,
        "org":        token.claims.org,
        "roles":      token.claims.roles,
    })))
}

// ── POST /api/v1/admin/users/enroll ──────────────────────────────────────────
// Requires ADMIN role. Issues a PQ cert for a new user, stores their
// password-encrypted keypair, and updates the in-memory cert registry so they
// can log in immediately without a node restart.

#[derive(Deserialize)]
pub struct EnrollRequest {
    pub username: String,
    pub password: String,
    pub roles: Vec<String>,
    pub org: Option<String>,
    pub email: Option<String>,
    /// Certificate validity in days (default: 365)
    pub days: Option<u64>,
}

pub async fn admin_enroll_user(
    State(state): State<Arc<AppState>>,
    Extension(caller): Extension<CallerIdentity>,
    Json(body): Json<EnrollRequest>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&caller)?;

    let ca = state.ca.as_ref().ok_or_else(|| {
        ApiError::Internal(
            "CA not configured — start the node with QORVUM_CA_PASSPHRASE to enable enrollment"
                .into(),
        )
    })?;
    let user_store = state.user_store.as_ref().ok_or_else(|| {
        ApiError::Internal("User store not available".into())
    })?;

    if user_store.exists(&body.username) {
        return Err(ApiError::BadRequest(format!(
            "User '{}' already exists",
            body.username
        )));
    }

    let org = body
        .org
        .clone()
        .unwrap_or_else(|| caller.org.clone());

    let subject = CertSubject {
        common_name: body.username.clone(),
        org,
        org_unit: None,
        email: body.email.clone(),
    };

    let (cert, keypair) = {
        let mut ca_guard = ca.lock().await;
        ca_guard
            .issue_user_cert(subject, body.roles.clone(), body.days.unwrap_or(365))
            .map_err(|e| ApiError::Internal(format!("Failed to issue certificate: {}", e)))?
    };

    user_store
        .enroll(&cert, &keypair, &body.password)
        .map_err(|e| ApiError::Internal(format!("Failed to store user credentials: {}", e)))?;

    // Hot-update verifier so the new user can log in without a node restart
    if let Some(verifier_lock) = state.verifier.as_ref() {
        verifier_lock.write().await.add_cert(cert.clone());
    }

    Ok(ok(json!({
        "username":         cert.subject.common_name,
        "org":              cert.subject.org,
        "roles":            cert.roles,
        "cert_fingerprint": hex::encode(cert.fingerprint()),
        "expires_at":       cert.not_after / 1_000_000_000,
        "message":          "User enrolled. Use POST /api/v1/auth/login to get a token.",
    })))
}

// ── GET /api/v1/admin/users ───────────────────────────────────────────────────

pub async fn admin_list_users(
    State(state): State<Arc<AppState>>,
    Extension(caller): Extension<CallerIdentity>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&caller)?;

    let user_store = state.user_store.as_ref().ok_or_else(|| {
        ApiError::Internal("User store not available".into())
    })?;

    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    // Load CRL so revoked users show the correct status
    let crl = if let Some(ca) = state.ca.as_ref() {
        let ca_guard = ca.lock().await;
        ca_guard.export_public().crl
    } else {
        std::collections::HashMap::new()
    };

    let users: Vec<Value> = user_store
        .list_usernames()
        .into_iter()
        .filter_map(|name| {
            let cert = user_store.get_cert(&name).ok()?;
            let serial_hex = hex::encode(cert.serial);
            let status = if crl.contains_key(&serial_hex) {
                "REVOKED"
            } else if cert.not_after < now_nanos {
                "EXPIRED"
            } else {
                "VALID"
            };
            Some(json!({
                "username":   name,
                "org":        cert.subject.org,
                "roles":      cert.roles,
                "expires_at": cert.not_after / 1_000_000_000,
                "status":     status,
            }))
        })
        .collect();

    let total = users.len();
    Ok(ok(json!({ "users": users, "total": total })))
}

// ── POST /api/v1/admin/users/:username/revoke ─────────────────────────────────

#[derive(Deserialize, Serialize)]
pub struct RevokeRequest {
    pub reason: Option<String>,
}

pub async fn admin_revoke_user(
    State(state): State<Arc<AppState>>,
    Extension(caller): Extension<CallerIdentity>,
    Path(username): Path<String>,
    Json(body): Json<RevokeRequest>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&caller)?;

    let ca = state.ca.as_ref().ok_or_else(|| {
        ApiError::Internal("CA not configured".into())
    })?;
    let user_store = state.user_store.as_ref().ok_or_else(|| {
        ApiError::Internal("User store not available".into())
    })?;

    let cert = user_store
        .get_cert(&username)
        .map_err(|_| ApiError::NotFound(format!("User '{}' not found", username)))?;

    let reason = body.reason.unwrap_or_else(|| "Revoked by admin".into());

    {
        let mut ca_guard = ca.lock().await;
        ca_guard
            .revoke(cert.serial, &reason)
            .map_err(|e| ApiError::Internal(format!("Revocation failed: {}", e)))?;
    }

    // Hot-update in-memory CRL so existing tokens are rejected immediately
    if let Some(verifier_lock) = state.verifier.as_ref() {
        verifier_lock
            .write()
            .await
            .add_revocation(cert.serial, reason.clone());
    }

    Ok(ok(json!({
        "username": username,
        "serial":   hex::encode(cert.serial),
        "reason":   reason,
        "message":  "User revoked. Existing tokens will be rejected immediately.",
    })))
}

// ── GET /api/v1/admin/certs ───────────────────────────────────────────────────
// Federation view: aggregates certificates from ALL trusted CAs (all orgs).
// Falls back to local CA only if no verifier is configured.
// Requires ADMIN role.

pub async fn admin_list_certs(
    State(state): State<Arc<AppState>>,
    Extension(caller): Extension<CallerIdentity>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&caller)?;

    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    fn cert_to_json(cert: &qorvum_msp::PQCertificate, crl: &std::collections::HashMap<String, String>, now_nanos: u64, ca_name: &str) -> Value {
        let serial_hex = hex::encode(cert.serial);
        let status = if crl.contains_key(&serial_hex) {
            "REVOKED"
        } else if cert.not_after < now_nanos {
            "EXPIRED"
        } else {
            "VALID"
        };
        json!({
            "serial":        serial_hex,
            "subject":       cert.subject.common_name,
            "org":           cert.subject.org,
            "org_unit":      cert.subject.org_unit,
            "email":         cert.subject.email,
            "issuer":        ca_name,
            "cert_type":     cert.cert_type.to_string(),
            "algorithm":     format!("{:?}", cert.algorithm),
            "roles":         cert.roles,
            "not_before":    cert.not_before / 1_000_000_000,
            "not_after":     cert.not_after  / 1_000_000_000,
            "fingerprint":   hex::encode(cert.fingerprint()),
            "status":        status,
            "revoke_reason": crl.get(&serial_hex),
        })
    }

    // Prefer verifier (has all trusted CAs from all orgs) over local CA only
    if let Some(verifier_lock) = state.verifier.as_ref() {
        let verifier = verifier_lock.read().await;
        let trusted = verifier.trusted_cas();

        let mut all_certs: Vec<Value> = Vec::new();
        let mut all_crl: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut ca_list: Vec<Value> = Vec::new();

        for ca_pub in trusted {
            // Aggregate certs from this CA
            for cert in ca_pub.cert_registry.values() {
                all_certs.push(cert_to_json(cert, &ca_pub.crl, now_nanos, &ca_pub.ca_name));
            }
            // Merge CRLs
            all_crl.extend(ca_pub.crl.clone());

            let ca_cert = &ca_pub.ca_cert;
            ca_list.push(json!({
                "name":        ca_pub.ca_name,
                "org":         ca_pub.ca_org,
                "serial":      hex::encode(ca_cert.serial),
                "algorithm":   format!("{:?}", ca_cert.algorithm),
                "not_before":  ca_cert.not_before / 1_000_000_000,
                "not_after":   ca_cert.not_after  / 1_000_000_000,
                "fingerprint": hex::encode(ca_cert.fingerprint()),
            }));
        }

        let total = all_certs.len();
        return Ok(ok(json!({
            "certs":  all_certs,
            "crl":    all_crl,
            "cas":    ca_list,
            // Keep "ca" as first CA for backwards compat with single-CA frontend
            "ca":     ca_list.first(),
            "total":  total,
        })));
    }

    // Fallback: local CA only (no verifier configured — dev mode)
    let ca = state.ca.as_ref().ok_or_else(|| {
        ApiError::Internal("CA not configured — start the node with QORVUM_CA_PASSPHRASE".into())
    })?;
    let public = {
        let ca_guard = ca.lock().await;
        ca_guard.export_public()
    };

    let certs: Vec<Value> = public.cert_registry.values()
        .map(|cert| cert_to_json(cert, &public.crl, now_nanos, &public.ca_name))
        .collect();

    let ca_cert = &public.ca_cert;
    let ca_info = json!({
        "name":        public.ca_name,
        "org":         public.ca_org,
        "serial":      hex::encode(ca_cert.serial),
        "algorithm":   format!("{:?}", ca_cert.algorithm),
        "not_before":  ca_cert.not_before / 1_000_000_000,
        "not_after":   ca_cert.not_after  / 1_000_000_000,
        "fingerprint": hex::encode(ca_cert.fingerprint()),
    });

    let total = certs.len();
    Ok(ok(json!({
        "certs": certs,
        "crl":   public.crl,
        "cas":   [ca_info.clone()],
        "ca":    ca_info,
        "total": total,
    })))
}
