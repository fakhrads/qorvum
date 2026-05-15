//! qv — Qorvum CLI tool

use anyhow::{bail, Context, Result};
use chrono::{TimeZone, Utc};
use clap::{Parser, Subcommand};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use qorvum_msp::{
    ca::CertificateAuthority,
    certificate::{CertSubject, PQCertificate},
    identity::Identity,
    token::QorvumToken,
    verifier::IdentityVerifier,
};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

// ── Direktori default ─────────────────────────────────────────────────────────

/// Root direktori Qorvum di home user: ~/.qorvum
fn qorvum_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".qorvum")
}

/// Direktori CA default: ~/.qorvum/ca/<org_name>
fn default_ca_dir(org: &str) -> PathBuf {
    qorvum_home()
        .join("ca")
        .join(org.to_lowercase().replace(' ', "-"))
}

/// Direktori identitas default: ~/.qorvum/identities/
fn default_identities_dir() -> PathBuf {
    qorvum_home().join("identities")
}

/// Direktori CA lokal (--local): ./qorvum-pki/ca/<org_name>
fn local_ca_dir(org: &str) -> PathBuf {
    PathBuf::from("qorvum-pki")
        .join("ca")
        .join(org.to_lowercase().replace(' ', "-"))
}

/// Direktori identitas lokal (--local): ./qorvum-pki/identities/
fn local_identities_dir() -> PathBuf {
    PathBuf::from("qorvum-pki").join("identities")
}

/// Active profile path: ~/.qorvum/active.profile
fn profile_path() -> PathBuf {
    qorvum_home().join("active.profile")
}

// ── CLI top-level ─────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "qv",
    about = "Qorvum CLI — blockchain interaction & identity management",
    version
)]
struct Cli {
    #[arg(long, default_value = "http://localhost:8080", env = "QORVUM_URL")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Certificate Authority management
    Ca {
        #[command(subcommand)]
        action: CaCommands,
    },
    /// Local identity management
    Identity {
        #[command(subcommand)]
        action: IdentityCommands,
    },
    /// Node management utilities
    Node {
        #[command(subcommand)]
        action: NodeCommands,
    },
    /// Invoke a contract function (write transaction)
    Invoke {
        contract: String,
        function: String,
        #[arg(short, long)]
        args: String,
    },
    /// Query a contract function (read-only)
    Query {
        contract: String,
        function: String,
        #[arg(short, long, default_value = "{}")]
        args: String,
    },
    /// Get a record directly from the ledger
    Get {
        collection: String,
        partition:  String,
        id:         String,
    },
    /// List records in a collection
    List {
        collection: String,
        #[arg(short, long)]
        partition: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Get full history of a record
    History {
        collection: String,
        id:         String,
    },
    /// Get a block by number
    Block { number: u64 },
    /// Show node health
    Health,
}

#[derive(Subcommand, Debug)]
enum CaCommands {
    /// Initialize a new Certificate Authority
    Init {
        #[arg(long)]
        org: String,
        /// Simpan di folder project (./qorvum-pki/) alih-alih ~/.qorvum/
        #[arg(long)]
        local: bool,
        /// Override direktori output secara manual
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Issue a new certificate (user atau node)
    Issue {
        #[arg(long)]
        ca: Option<PathBuf>,
        #[arg(long)]
        org: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        roles: String,
        #[arg(long, default_value = "365")]
        days: u64,
        /// Simpan di folder project alih-alih ~/.qorvum/identities/
        #[arg(long)]
        local: bool,
        /// Override direktori output secara manual
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Revoke a certificate
    Revoke {
        #[arg(long)]
        ca: Option<PathBuf>,
        #[arg(long)]
        org: Option<String>,
        #[arg(long)]
        cert: PathBuf,
        #[arg(long, default_value = "Revoked by administrator")]
        reason: String,
    },
    /// List all issued certificates and their status
    List {
        #[arg(long)]
        ca: Option<PathBuf>,
        #[arg(long)]
        org: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    /// Set the active identity for CLI commands
    Use {
        cert_file: PathBuf,
        key_file:  PathBuf,
    },
    /// Show current active identity
    Show,
    /// Issue a short-lived bearer token from current identity
    Token {
        #[arg(long, default_value = "3600")]
        ttl: u64,
    },
    /// Verify a token or certificate against a CA
    Verify {
        token_or_cert: String,
        #[arg(long)]
        ca: Option<PathBuf>,
        #[arg(long)]
        org: Option<String>,
    },
    /// List all identities in ~/.qorvum/identities/
    List {
        #[arg(long)]
        local: bool,
    },
}

#[derive(Subcommand, Debug)]
enum NodeCommands {
    /// Print the Peer ID of this node from its saved keypair
    PeerId {
        /// Direktori data node (berisi validator.key)
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },
    /// Print node info (peer id, validator pubkey, data dir)
    Info {
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn passphrase_prompt(prompt: &str) -> Result<String> {
    rpassword::prompt_password(prompt).context("Failed to read passphrase")
}

fn nanos_to_date(nanos: u64) -> String {
    let secs = (nanos / 1_000_000_000) as i64;
    Utc.timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "?".to_string())
}

fn cert_status(cert: &PQCertificate, revoked: bool) -> &'static str {
    if revoked { return "REVOKED"; }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    if cert.is_valid_at(now) { "VALID" } else { "EXPIRED" }
}

/// Resolve CA directory dari kombinasi --ca, --org, atau default ~/.qorvum/ca/<org>
fn resolve_ca_dir(ca: Option<PathBuf>, org: Option<String>) -> Result<PathBuf> {
    if let Some(p) = ca {
        return Ok(p);
    }
    let org = org.context(
        "Perlu --org <ORG> atau --ca <DIR> untuk menemukan CA.\n\
         Contoh: qv ca list --org Org1"
    )?;
    let dir = default_ca_dir(&org);
    if !dir.exists() {
        bail!(
            "CA untuk org '{}' tidak ditemukan di {:?}\n\
             Jalankan: qv ca init --org {}",
            org, dir, org
        );
    }
    Ok(dir)
}

fn get_bearer_token() -> Result<String> {
    if let Ok(token) = std::env::var("QORVUM_TOKEN") {
        return Ok(token);
    }
    let identity = load_active_identity()?;
    let token = QorvumToken::issue(&identity, 3600)?;
    token.to_bearer_string().context("Failed to encode token")
}

// ── Active profile ────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ActiveProfile {
    cert_path: PathBuf,
    key_path:  PathBuf,
}

fn save_profile(cert_path: PathBuf, key_path: PathBuf) -> Result<()> {
    let profile = ActiveProfile {
        cert_path: cert_path.canonicalize().unwrap_or(cert_path),
        key_path:  key_path.canonicalize().unwrap_or(key_path),
    };
    let path = profile_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, serde_json::to_string_pretty(&profile)?)?;
    Ok(())
}

fn load_profile() -> Result<ActiveProfile> {
    let path = profile_path();
    if !path.exists() {
        bail!(
            "No active identity set.\n\
             Jalankan: qv identity use <CERT_FILE> <KEY_FILE>"
        );
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
}

fn load_active_identity() -> Result<Identity> {
    let profile = load_profile()?;
    Identity::load_unencrypted(&profile.cert_path, &profile.key_path)
        .context("Failed to load identity — check cert and key files")
}

// ── CA commands ───────────────────────────────────────────────────────────────

fn cmd_ca_init(
    org:        String,
    local:      bool,
    out:        Option<PathBuf>,
    passphrase: Option<String>,
) -> Result<()> {
    let out_dir = match out {
        Some(p) => p,
        None if local => local_ca_dir(&org),
        None => default_ca_dir(&org),
    };

    let pass = match passphrase {
        Some(p) => p,
        None => {
            let p1 = passphrase_prompt("CA passphrase: ")?;
            let p2 = passphrase_prompt("Confirm passphrase: ")?;
            if p1 != p2 { bail!("Passphrases do not match"); }
            p1
        }
    };

    let ca_name = format!("{}-CA", org);
    let _ca = CertificateAuthority::init(&ca_name, &org, &out_dir, &pass)?;

    println!("✓ CA keypair generated (Dilithium3)");
    println!("✓ CA self-signed certificate → {}/ca.cert", out_dir.display());
    println!("✓ CA private key (encrypted)  → {}/ca.key", out_dir.display());
    println!("✓ CA initialized for {}.", org);
    println!();
    println!("  Direktori CA : {}", out_dir.display());
    println!("  Bagikan      : {}/ca.cert  (ke semua node & klien)", out_dir.display());
    println!("  RAHASIA      : {}/ca.key   (jangan pernah dibagikan)", out_dir.display());

    Ok(())
}

fn cmd_ca_issue(
    ca:    Option<PathBuf>,
    org:   Option<String>,
    name:  String,
    roles: String,
    days:  u64,
    local: bool,
    out:   Option<PathBuf>,
) -> Result<()> {
    let ca_dir = resolve_ca_dir(ca, org.clone())?;
    let pass = passphrase_prompt("CA passphrase: ")?;
    let mut ca_obj = CertificateAuthority::load(&ca_dir, &pass)?;

    let role_list: Vec<String> = roles
        .split(',')
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();

    let subject = CertSubject {
        common_name: name.clone(),
        org:         ca_obj.org.clone(),
        org_unit:    None,
        email:       None,
    };

    let (cert, keypair) = ca_obj.issue_user_cert(subject, role_list, days)?;

    // Tentukan direktori output
    let out_dir = match out {
        Some(p) => p,
        None if local => local_identities_dir(),
        None => default_identities_dir(),
    };
    std::fs::create_dir_all(&out_dir)?;

    let cert_path = out_dir.join(format!("{}.cert", name));
    let key_path  = out_dir.join(format!("{}.key",  name));

    std::fs::write(&cert_path, cert.to_pem_like())?;

    let alg_byte: u8 = 0u8;
    let key_bytes = bincode::serialize(&(
        alg_byte,
        keypair.public_key().bytes.clone(),
        keypair.secret_bytes(),
    ))?;
    std::fs::write(&key_path, key_bytes)?;

    let expires = nanos_to_date(cert.not_after);
    println!("✓ Keypair generated for '{}'", name);
    println!("✓ Certificate → {}", cert_path.display());
    println!("✓ Private key  → {}", key_path.display());
    println!("✓ Valid for {} days (expires {})", days, expires);
    println!();
    println!("  Untuk set sebagai identitas aktif:");
    println!("  qv identity use {} {}", cert_path.display(), key_path.display());

    Ok(())
}

fn cmd_ca_revoke(
    ca:     Option<PathBuf>,
    org:    Option<String>,
    cert_path: PathBuf,
    reason: String,
) -> Result<()> {
    let ca_dir = resolve_ca_dir(ca, org)?;
    let pass = passphrase_prompt("CA passphrase: ")?;
    let mut ca_obj = CertificateAuthority::load(&ca_dir, &pass)?;

    let pem  = std::fs::read_to_string(&cert_path)?;
    let cert = PQCertificate::from_pem_like(&pem)?;
    let serial_hex = &hex::encode(cert.serial)[..8];

    ca_obj.revoke(cert.serial, &reason)?;

    println!("✓ Certificate '{}' (serial: {}) revoked", cert.subject.common_name, serial_hex);
    println!("✓ CRL updated → {}/crl.json", ca_dir.display());
    println!();
    println!("  Distribusikan CRL terbaru ke semua node:");
    println!("  scp {}/crl.json user@<NODE_IP>:/etc/qorvum/ca/", ca_dir.display());

    Ok(())
}

fn cmd_ca_list(ca: Option<PathBuf>, org: Option<String>) -> Result<()> {
    let ca_dir    = resolve_ca_dir(ca, org)?;
    let certs_dir = ca_dir.join("certs");

    if !certs_dir.exists() {
        println!("Belum ada sertifikat yang diterbitkan.");
        return Ok(());
    }

    let crl: std::collections::HashMap<String, String> = {
        let crl_path = ca_dir.join("crl.json");
        if crl_path.exists() {
            let json: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(crl_path)?)?;
            json["revoked"]
                .as_object()
                .map(|m| m.iter().map(|(k, v)| {
                    (k.clone(), v.as_str().unwrap_or("").to_string())
                }).collect())
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        }
    };

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Serial", "Roles", "Issued", "Expires", "Status"]);

    for entry in std::fs::read_dir(&certs_dir)?.flatten() {
        if let Ok(pem) = std::fs::read_to_string(entry.path()) {
            if let Ok(cert) = PQCertificate::from_pem_like(&pem) {
                let serial_hex = hex::encode(cert.serial);
                let short      = &serial_hex[..8];
                let is_revoked = crl.contains_key(&serial_hex);
                let status     = cert_status(&cert, is_revoked);
                let roles      = cert.roles.join(", ");
                table.add_row(vec![
                    cert.subject.common_name.clone(),
                    short.to_string(),
                    roles,
                    nanos_to_date(cert.not_before),
                    nanos_to_date(cert.not_after),
                    status.to_string(),
                ]);
            }
        }
    }

    println!("CA: {}", ca_dir.display());
    println!("{table}");
    Ok(())
}

// ── Identity commands ─────────────────────────────────────────────────────────

fn cmd_identity_use(cert_file: PathBuf, key_file: PathBuf) -> Result<()> {
    let identity = Identity::load_unencrypted(&cert_file, &key_file)
        .context("Failed to load identity — check cert and key paths")?;

    if !identity.cert.verify() {
        bail!("Certificate signature is invalid");
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    if !identity.cert.is_valid_at(now) {
        bail!("Certificate is expired or not yet valid");
    }

    save_profile(cert_file, key_file)?;

    let subject = &identity.cert.subject;
    let roles   = identity.cert.roles.join(", ");
    let expires = nanos_to_date(identity.cert.not_after);
    println!("✓ Identity set: {}@{} [{}]", subject.common_name, subject.org, roles);
    println!("✓ Certificate valid until {}", expires);
    println!("✓ Profile saved → {}", profile_path().display());

    Ok(())
}

fn cmd_identity_show() -> Result<()> {
    let identity = load_active_identity()?;
    let subject  = &identity.cert.subject;
    let roles    = identity.cert.roles.join(", ");
    let expires  = nanos_to_date(identity.cert.not_after);
    let fp       = hex::encode(identity.cert.fingerprint());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let status = if identity.cert.is_valid_at(now) { "VALID" } else { "EXPIRED" };

    println!("Subject     : {}@{}", subject.common_name, subject.org);
    println!("Roles       : [{}]", roles);
    println!("Type        : {}", identity.cert.cert_type);
    println!("Expires     : {} ({})", expires, status);
    println!("Fingerprint : {}", fp);

    Ok(())
}

fn cmd_identity_token(ttl: u64) -> Result<()> {
    let identity = load_active_identity()?;
    let token    = QorvumToken::issue(&identity, ttl)?;
    let bearer   = token.to_bearer_string()?;
    println!("{}", bearer);
    Ok(())
}

fn cmd_identity_verify(
    token_or_cert: String,
    ca:            Option<PathBuf>,
    org:           Option<String>,
) -> Result<()> {
    let ca_dir   = resolve_ca_dir(ca, org)?;
    let verifier = IdentityVerifier::new(&[ca_dir])?;

    let content = if std::path::Path::new(&token_or_cert).exists() {
        std::fs::read_to_string(&token_or_cert)
            .context("Failed to read file")?
            .trim()
            .to_string()
    } else {
        token_or_cert.clone()
    };

    if content.starts_with("-----BEGIN") {
        match PQCertificate::from_pem_like(&content) {
            Ok(cert) => match verifier.verify_cert(&cert) {
                Ok(()) => {
                    println!("✓ Certificate VALID");
                    println!("  Subject : {}@{}", cert.subject.common_name, cert.subject.org);
                    println!("  Roles   : [{}]", cert.roles.join(", "));
                    println!("  Expires : {}", nanos_to_date(cert.not_after));
                }
                Err(e) => println!("✗ Certificate INVALID: {}", e),
            },
            Err(_) => bail!("Could not parse PEM certificate"),
        }
    } else {
        match verifier.verify_token(&content) {
            Ok(id) => {
                println!("✓ Token VALID");
                println!("  Subject : {}@{}", id.subject, id.org);
                println!("  Roles   : [{}]", id.roles.join(", "));
                println!("  Type    : {}", id.cert_type);
            }
            Err(e) => println!("✗ Token INVALID: {}", e),
        }
    }
    Ok(())
}

fn cmd_identity_list(local: bool) -> Result<()> {
    let dir = if local {
        local_identities_dir()
    } else {
        default_identities_dir()
    };

    if !dir.exists() {
        println!("Belum ada identitas tersimpan di {}", dir.display());
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Name", "Org", "Roles", "Expires", "Status"]);

    for entry in std::fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("cert") {
            if let Ok(pem) = std::fs::read_to_string(&path) {
                if let Ok(cert) = PQCertificate::from_pem_like(&pem) {
                    let status = if cert.is_valid_at(now) { "VALID" } else { "EXPIRED" };
                    table.add_row(vec![
                        cert.subject.common_name.clone(),
                        cert.subject.org.clone(),
                        cert.roles.join(", "),
                        nanos_to_date(cert.not_after),
                        status.to_string(),
                    ]);
                }
            }
        }
    }

    println!("Identitas di: {}", dir.display());
    println!("{table}");
    Ok(())
}

// ── Node commands ─────────────────────────────────────────────────────────────

fn cmd_node_peer_id(data_dir: PathBuf) -> Result<()> {
    use qorvum_crypto::signing::{PQKeypair, SigningAlgorithm};

    let key_path = data_dir.join("validator.key");

    if !key_path.exists() {
        bail!(
            "Validator key tidak ditemukan di {:?}\n\
             Node harus dijalankan minimal sekali dulu untuk generate keypair.\n\
             Atau jalankan: cargo run -p qorvum-node -- --data-dir {} --role validator\n\
             lalu Ctrl+C setelah keypair terbuat.",
            key_path,
            data_dir.display()
        );
    }

    let bytes = std::fs::read(&key_path)
        .context("Failed to read validator key")?;

    let (alg_byte, pk_bytes, sk_bytes): (u8, Vec<u8>, Vec<u8>) =
        bincode::deserialize(&bytes)
            .context("Validator key file corrupt — hapus dan biarkan node regenerate")?;

    let algorithm = if alg_byte == 0 {
        SigningAlgorithm::Dilithium3
    } else {
        SigningAlgorithm::Falcon512
    };

    let keypair = PQKeypair::from_bytes(algorithm, pk_bytes, sk_bytes);

    // Derive libp2p Peer ID dari validator pubkey
    // Peer ID = base58(multihash(SHA256(pubkey_bytes)))
    // Kita gunakan blake3 hash untuk derive ID yang konsisten
    let pubkey_hash = qorvum_crypto::hash(&keypair.public_key().bytes);
    let peer_id_hex = hex::encode(&pubkey_hash[..20]); // 20 bytes = 40 hex chars

    // Format sebagai multiaddr-compatible peer id representation
    // Dalam libp2p sebenarnya dari ed25519 keypair, tapi kita tampilkan
    // validator pubkey hex yang dipakai di --validator-keys
    println!("Data dir      : {}", data_dir.display());
    println!("Validator key : {}", key_path.display());
    println!();
    println!("Validator pubkey (untuk --validator-keys di node lain):");
    println!("{}", keypair.public_key().to_hex());
    println!();
    println!("Node identity hash (untuk referensi):");
    println!("{}", peer_id_hex);
    println!();
    println!("Untuk bootstrap peers, format multiaddr:");
    println!("/ip4/<IP_SERVER>/tcp/7051/p2p/<LIBP2P_PEER_ID>");
    println!();
    println!("Catatan: libp2p Peer ID yang sebenarnya tampil di log saat node start:");
    println!("  INFO  Local peer id: 12D3KooW...");

    Ok(())
}

fn cmd_node_info(data_dir: PathBuf) -> Result<()> {
    use qorvum_crypto::signing::{PQKeypair, SigningAlgorithm};

    let key_path = data_dir.join("validator.key");

    println!("=== Qorvum Node Info ===");
    println!("Data dir : {}", data_dir.display());

    if !key_path.exists() {
        println!("Validator key : (belum ada — jalankan node sekali untuk generate)");
        return Ok(());
    }

    let bytes = std::fs::read(&key_path)?;
    if let Ok((alg_byte, pk_bytes, sk_bytes)) =
        bincode::deserialize::<(u8, Vec<u8>, Vec<u8>)>(&bytes)
    {
        let algorithm = if alg_byte == 0 {
            SigningAlgorithm::Dilithium3
        } else {
            SigningAlgorithm::Falcon512
        };
        let keypair = PQKeypair::from_bytes(algorithm, pk_bytes, sk_bytes);
        let pubkey  = keypair.public_key().to_hex();

        println!("Algorithm    : {:?}", algorithm);
        println!("Validator key: {}...{}", &pubkey[..16], &pubkey[pubkey.len()-16..]);
        println!("Full pubkey  :");
        println!("{}", pubkey);
    } else {
        println!("Validator key : (corrupt — hapus dan biarkan node regenerate)");
    }

    // Cek apakah ada ledger
    let ledger_path = data_dir.join("ledger");
    if ledger_path.exists() {
        println!("Ledger       : {} (ada)", ledger_path.display());
    } else {
        println!("Ledger       : (belum ada)");
    }

    Ok(())
}

// ── Gateway API commands ──────────────────────────────────────────────────────

async fn send_api(
    client:  &reqwest::Client,
    req:     reqwest::RequestBuilder,
    bearer:  Option<String>,
) -> Result<serde_json::Value> {
    let req = match bearer {
        Some(token) => req.header("Authorization", format!("Bearer {}", token)),
        None => req,
    };
    let resp = req.send().await.context("Request failed")?;
    let json: serde_json::Value = resp.json().await.context("Failed to parse response")?;
    Ok(json)
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("warn"))
        .with_target(false)
        .init();

    let result = run(cli).await;
    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        // ── CA ────────────────────────────────────────────────────────────────
        Commands::Ca { action } => match action {
            CaCommands::Init { org, local, out, passphrase } => {
                cmd_ca_init(org, local, out, passphrase)?;
            }
            CaCommands::Issue { ca, org, name, roles, days, local, out } => {
                cmd_ca_issue(ca, org, name, roles, days, local, out)?;
            }
            CaCommands::Revoke { ca, org, cert, reason } => {
                cmd_ca_revoke(ca, org, cert, reason)?;
            }
            CaCommands::List { ca, org } => {
                cmd_ca_list(ca, org)?;
            }
        },

        // ── Identity ──────────────────────────────────────────────────────────
        Commands::Identity { action } => match action {
            IdentityCommands::Use { cert_file, key_file } => {
                cmd_identity_use(cert_file, key_file)?;
            }
            IdentityCommands::Show => cmd_identity_show()?,
            IdentityCommands::Token { ttl } => cmd_identity_token(ttl)?,
            IdentityCommands::Verify { token_or_cert, ca, org } => {
                cmd_identity_verify(token_or_cert, ca, org)?;
            }
            IdentityCommands::List { local } => cmd_identity_list(local)?,
        },

        // ── Node ──────────────────────────────────────────────────────────────
        Commands::Node { action } => match action {
            NodeCommands::PeerId { data_dir } => cmd_node_peer_id(data_dir)?,
            NodeCommands::Info   { data_dir } => cmd_node_info(data_dir)?,
        },

        // ── Gateway API ───────────────────────────────────────────────────────
        Commands::Invoke { contract, function, args } => {
            let body: serde_json::Value =
                serde_json::from_str(&args).context("Args must be valid JSON")?;
            let bearer = get_bearer_token().ok();
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let req    = client
                .post(format!("{}/api/v1/invoke/{}/{}", base, contract, function))
                .json(&body);
            let json = send_api(&client, req, bearer).await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        Commands::Query { contract, function, args } => {
            let bearer = get_bearer_token().ok();
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let req    = client
                .get(format!("{}/api/v1/query/{}/{}", base, contract, function))
                .query(&[("args", &args)]);
            let json = send_api(&client, req, bearer).await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        Commands::Get { collection, partition, id } => {
            let bearer = get_bearer_token().ok();
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let req    = client.get(format!(
                "{}/api/v1/records/{}/{}/{}",
                base, collection, partition, id
            ));
            let json = send_api(&client, req, bearer).await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        Commands::List { collection, partition, limit } => {
            let bearer = get_bearer_token().ok();
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let mut req = client
                .get(format!("{}/api/v1/records/{}", base, collection))
                .query(&[("limit", limit.to_string())]);
            if let Some(p) = partition {
                req = req.query(&[("partition", p)]);
            }
            let json = send_api(&client, req, bearer).await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        Commands::History { collection, id } => {
            let bearer = get_bearer_token().ok();
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let req    = client.get(format!(
                "{}/api/v1/history/{}/{}", base, collection, id
            ));
            let json = send_api(&client, req, bearer).await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        Commands::Block { number } => {
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let resp   = client
                .get(format!("{}/api/v1/blocks/{}", base, number))
                .send().await.context("Request failed")?;
            let json: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        Commands::Health => {
            let client = reqwest::Client::new();
            let base   = cli.url.trim_end_matches('/');
            let resp   = client
                .get(format!("{}/api/v1/health", base))
                .send().await.context("Request failed")?;
            let json: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }
    Ok(())
}