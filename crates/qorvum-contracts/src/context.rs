//! ChainContextImpl — the concrete implementation of chain-sdk's ChainContext trait.
//! Wraps SimulationContext and bridges it to the contracts.

use chain_sdk::{ChainContext, ChainError, HistoryEntry, Identity, Pagination,
                QueryResult, SortBy, UpsertAction};
use chain_sdk::types::Filter as SdkFilter;
use chain_sdk::types::FieldValue as SdkFieldValue;
use qorvum_ledger::record::FieldValue as LedgerFieldValue;
use qorvum_ledger::query::{Filter as LedgerFilter, QueryEngine, apply_filter, has_include_deleted};
use qorvum_ledger::store::LedgerStore;
use qorvum_ledger::world_state::{SimulationContext, PendingWrite};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ChainContextImpl {
    pub sim:    SimulationContext,
    pub engine: QueryEngine,
    roles:      Vec<String>,
    verified:   bool,
}

impl ChainContextImpl {
    /// Standard constructor — verified defaults to false (dev mode / existing tests).
    pub fn new(
        store:        Arc<dyn LedgerStore>,
        caller_id:    String,
        caller_msp:   String,
        caller_roles: Vec<String>,
        tx_id:        [u8; 32],
        timestamp:    u64,
    ) -> Self {
        Self::new_with_verified(store, caller_id, caller_msp, caller_roles, tx_id, timestamp, false)
    }

    /// Full constructor with explicit verified flag for gateway use.
    pub fn new_with_verified(
        store:        Arc<dyn LedgerStore>,
        caller_id:    String,
        caller_msp:   String,
        caller_roles: Vec<String>,
        tx_id:        [u8; 32],
        timestamp:    u64,
        verified:     bool,
    ) -> Self {
        let sim    = SimulationContext::new(
            store.clone(), caller_id, caller_msp, caller_roles.clone(), tx_id, timestamp,
        );
        let engine = QueryEngine::new(store);
        Self { sim, engine, roles: caller_roles, verified }
    }

    pub fn identity(&self) -> Identity {
        self.to_identity()
    }

    fn to_identity(&self) -> Identity {
        let caller_id = self.sim.caller_id();
        let subject = caller_id
            .split('@')
            .next()
            .unwrap_or(caller_id)
            .to_string();
        let org = self.sim.caller_msp().to_string();
        Identity {
            id:        caller_id.to_string(),
            msp_id:    org.clone(),
            roles:     self.roles.clone(),
            subject,
            org,
            cert_type: "User".to_string(),
            email:     None,
            verified:  self.verified,
        }
    }
}

// ── Field value conversions ───────────────────────────────────────────────────

fn sdk_to_ledger(v: SdkFieldValue) -> LedgerFieldValue {
    match v {
        SdkFieldValue::Null         => LedgerFieldValue::Null,
        SdkFieldValue::Bool(b)      => LedgerFieldValue::Bool(b),
        SdkFieldValue::Int(n)       => LedgerFieldValue::Int(n),
        SdkFieldValue::Float(f)     => LedgerFieldValue::Float(f),
        SdkFieldValue::Text(s)      => LedgerFieldValue::Text(s),
        SdkFieldValue::Bytes(b)     => LedgerFieldValue::Bytes(b),
        SdkFieldValue::Timestamp(t) => LedgerFieldValue::Timestamp(t),
        SdkFieldValue::Json(j)      => LedgerFieldValue::Json(j),
    }
}

fn ledger_to_sdk(v: LedgerFieldValue) -> SdkFieldValue {
    match v {
        LedgerFieldValue::Null         => SdkFieldValue::Null,
        LedgerFieldValue::Bool(b)      => SdkFieldValue::Bool(b),
        LedgerFieldValue::Int(n)       => SdkFieldValue::Int(n),
        LedgerFieldValue::Float(f)     => SdkFieldValue::Float(f),
        LedgerFieldValue::Text(s)      => SdkFieldValue::Text(s),
        LedgerFieldValue::Bytes(b)     => SdkFieldValue::Bytes(b),
        LedgerFieldValue::Timestamp(t) => SdkFieldValue::Timestamp(t),
        LedgerFieldValue::Json(j)      => SdkFieldValue::Json(j),
    }
}

fn convert_fields(fields: HashMap<String, SdkFieldValue>) -> HashMap<String, LedgerFieldValue> {
    fields.into_iter().map(|(k, v)| (k, sdk_to_ledger(v))).collect()
}

fn sdk_to_ledger_filter(f: SdkFilter) -> LedgerFilter {
    match f {
        SdkFilter::Eq(k, v)          => LedgerFilter::Eq(k, sdk_to_ledger(v)),
        SdkFilter::Neq(k, v)         => LedgerFilter::Neq(k, sdk_to_ledger(v)),
        SdkFilter::Gt(k, v)          => LedgerFilter::Gt(k, sdk_to_ledger(v)),
        SdkFilter::Lt(k, v)          => LedgerFilter::Lt(k, sdk_to_ledger(v)),
        SdkFilter::Gte(k, v)         => LedgerFilter::Gte(k, sdk_to_ledger(v)),
        SdkFilter::Lte(k, v)         => LedgerFilter::Lte(k, sdk_to_ledger(v)),
        SdkFilter::Between(k, lo, hi)=> LedgerFilter::And(vec![
            LedgerFilter::Gte(k.clone(), sdk_to_ledger(lo)),
            LedgerFilter::Lte(k, sdk_to_ledger(hi)),
        ]),
        SdkFilter::In(k, vs)         => LedgerFilter::In(k, vs.into_iter().map(sdk_to_ledger).collect()),
        SdkFilter::IsNull(k)         => LedgerFilter::IsNull(k),
        SdkFilter::IsNotNull(k)      => LedgerFilter::IsNotNull(k),
        SdkFilter::And(fs)           => LedgerFilter::And(fs.into_iter().map(sdk_to_ledger_filter).collect()),
        SdkFilter::Or(fs)            => LedgerFilter::Or(fs.into_iter().map(sdk_to_ledger_filter).collect()),
        SdkFilter::Not(f)            => LedgerFilter::Not(Box::new(sdk_to_ledger_filter(*f))),
        SdkFilter::IncludeDeleted    => LedgerFilter::IncludeDeleted,
    }
}

fn ledger_err(e: qorvum_ledger::LedgerError) -> ChainError {
    use qorvum_ledger::LedgerError::*;
    match e {
        NotFound(s)           => ChainError::NotFound(s),
        AlreadyExists(s)      => ChainError::AlreadyExists(s),
        ValidationFailed(s)   => ChainError::ValidationFailed(s),
        StorageError(s) | SerializationError(s) | BlockError(s) | DeltaError(s)
                              => ChainError::InternalError(s),
        HashMismatch          => ChainError::InternalError("delta hash mismatch".into()),
    }
}

impl ChainContext for ChainContextImpl {
    fn insert(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        fields:     HashMap<String, SdkFieldValue>,
    ) -> Result<serde_json::Value, ChainError> {
        let rec = self.sim.insert(collection, partition, id, convert_fields(fields))
            .map_err(ledger_err)?;
        Ok(serde_json::to_value(&rec).unwrap())
    }

    fn get(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
    ) -> Result<Option<serde_json::Value>, ChainError> {
        self.sim.get(collection, partition, id)
            .map(|opt| opt.map(|r| serde_json::to_value(&r).unwrap()))
            .map_err(ledger_err)
    }

    fn update(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        fields:     HashMap<String, SdkFieldValue>,
    ) -> Result<serde_json::Value, ChainError> {
        let rec = self.sim.update(collection, partition, id, convert_fields(fields))
            .map_err(ledger_err)?;
        Ok(serde_json::to_value(&rec).unwrap())
    }

    fn patch(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        patches:    HashMap<String, SdkFieldValue>,
    ) -> Result<serde_json::Value, ChainError> {
        let rec = self.sim.patch(collection, partition, id, convert_fields(patches))
            .map_err(ledger_err)?;
        Ok(serde_json::to_value(&rec).unwrap())
    }

    fn delete(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        reason:     Option<String>,
    ) -> Result<(), ChainError> {
        self.sim.soft_delete(collection, partition, id, reason)
            .map_err(ledger_err)
    }

    fn restore(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
    ) -> Result<serde_json::Value, ChainError> {
        let rec = self.sim.restore(collection, partition, id)
            .map_err(ledger_err)?;
        Ok(serde_json::to_value(&rec).unwrap())
    }

    fn query(
        &self,
        collection: &str,
        partition:  Option<&str>,
        filter:     Option<chain_sdk::types::Filter>,
        sort:       Option<Vec<SortBy>>,
        pagination: Option<Pagination>,
    ) -> Result<QueryResult, ChainError> {
        use qorvum_ledger::query::{Pagination as LP, SortBy as LS};

        let lf = filter.map(sdk_to_ledger_filter);
        let ls: Option<Vec<LS>> = sort.map(|sv| sv.into_iter()
            .map(|s| LS { field: s.field, descending: s.descending })
            .collect());
        let lp = pagination.map(|p| LP { limit: p.limit as usize, offset: 0 });

        let mut result = self.engine.query(
            collection, partition, lf.as_ref(), ls.as_deref(), lp.as_ref()
        ).map_err(ledger_err)?;

        // Merge pending writes for Read-Your-Own-Writes consistency
        let pending = self.sim.get_pending_writes();
        for w in pending {
            match w {
                PendingWrite::Put(r) => {
                    let key = r.composite_key();
                    if r.meta.collection == collection && (partition.is_none() || Some(r.meta.partition.as_str()) == partition) {
                        let include_deleted = lf.as_ref().map(|f| has_include_deleted(f)).unwrap_or(false);
                        let matches = (include_deleted || !r.meta.is_deleted) && 
                                     lf.as_ref().map(|f| apply_filter(&r, f)).unwrap_or(true);
                        
                        let existing_idx = result.records.iter().position(|rec| rec.composite_key() == key);
                        if matches {
                            if let Some(idx) = existing_idx {
                                result.records[idx] = r;
                            } else {
                                result.records.push(r);
                                result.total += 1;
                            }
                        } else if let Some(idx) = existing_idx {
                            result.records.remove(idx);
                            result.total -= 1;
                        }
                    }
                }
                PendingWrite::Delete(key) => {
                    if let Some(idx) = result.records.iter().position(|rec| rec.composite_key() == key) {
                        result.records.remove(idx);
                        result.total -= 1;
                    }
                }
            }
        }

        let records = result.records.into_iter()
            .map(|r| serde_json::to_value(&r).unwrap())
            .collect();

        Ok(QueryResult { records, total: result.total as u64, page_token: None })
    }

    fn get_history(
        &self,
        collection: &str,
        id:         &str,
    ) -> Result<Vec<HistoryEntry>, ChainError> {
        let entries = self.sim.get_history(collection, id)
            .map_err(ledger_err)?;
        Ok(entries.into_iter().map(|(ver, block_num)| HistoryEntry {
            version:    ver,
            tx_id:      [0u8; 32],
            block_num,
            timestamp:  0,
            updated_by: String::new(),
            fields:     HashMap::new(),
            is_deleted: false,
        }).collect())
    }

    fn batch_insert(
        &self,
        collection: &str,
        partition:  &str,
        records:    Vec<(String, HashMap<String, SdkFieldValue>)>,
    ) -> Result<Vec<serde_json::Value>, ChainError> {
        records.into_iter().map(|(id, fields)| {
            self.insert(collection, partition, &id, fields)
        }).collect()
    }

    fn upsert(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        fields:     HashMap<String, SdkFieldValue>,
    ) -> Result<(serde_json::Value, UpsertAction), ChainError> {
        let key = format!("{}~{}~{}", collection, partition, id);
        let exists = self.sim.get(collection, partition, id)
            .map_err(ledger_err)?.is_some();
        if exists {
            let rec = self.update(collection, partition, id, fields)?;
            Ok((rec, UpsertAction::Updated))
        } else {
            let rec = self.insert(collection, partition, id, fields)?;
            Ok((rec, UpsertAction::Inserted))
        }
    }

    fn emit_event(&self, name: &str, payload: &[u8]) {
        self.sim.emit_event(name, payload.to_vec());
    }

    fn caller_identity(&self) -> &Identity {
        // Safety: we keep identity in sim, return static ref
        // In production use Arc<Identity>
        Box::leak(Box::new(self.to_identity()))
    }

    fn has_role(&self, role: &str) -> bool {
        self.sim.has_role(role)
    }
}
