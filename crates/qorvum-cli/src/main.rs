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

// TUI imports
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Terminal,
};

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
    /// Interactive setup wizard — creates CA, admin identity, and config/node.yml
    Init {
        /// Organization name (skip wizard prompt if provided)
        #[arg(long)]
        org: Option<String>,
    },
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
    /// Smart contract management
    Contract {
        #[command(subcommand)]
        action: ContractCommands,
    },
    /// Get a block by number
    Block { number: u64 },
    /// Show node health
    Health,
}

#[derive(Subcommand, Debug)]
enum ContractCommands {
    /// Deploy a WASM contract to the node (requires ADMIN role)
    Deploy {
        /// Contract identifier (e.g. "todo-as")
        #[arg(long)]
        name: String,
        /// Path to the compiled .wasm file
        #[arg(long)]
        wasm: PathBuf,
    },
    /// List all deployed contracts
    List,
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
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },
    /// Print node info (peer id, validator pubkey, data dir)
    Info {
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
    },
    /// Live node dashboard — like `top` for your Qorvum node
    Top,
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
    _client: &reqwest::Client,
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

// ── Init wizard ───────────────────────────────────────────────────────────────

fn cmd_init(org_arg: Option<String>) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

    let theme = ColorfulTheme::default();

    println!();
    println!("  ╔══════════════════════════════════════════════╗");
    println!("  ║   Qorvum Setup Wizard                        ║");
    println!("  ╚══════════════════════════════════════════════╝");
    println!();

    // ── Step 1: Org name ──────────────────────────────────────────────────────
    let org: String = match org_arg {
        Some(o) => o,
        None => Input::with_theme(&theme)
            .with_prompt("Organization name")
            .default("MyOrg".to_string())
            .interact_text()?,
    };

    // ── Step 2: Admin username ────────────────────────────────────────────────
    let admin_name: String = Input::with_theme(&theme)
        .with_prompt("Admin username")
        .default("admin".to_string())
        .interact_text()?;

    // ── Step 3: Node role ─────────────────────────────────────────────────────
    let roles_list = &["all (validator + gateway + peer)", "validator", "gateway", "peer"];
    let role_idx = Select::with_theme(&theme)
        .with_prompt("Node role")
        .items(roles_list)
        .default(0)
        .interact()?;
    let node_role = ["all", "validator", "gateway", "peer"][role_idx];

    // ── Step 4: Addresses ─────────────────────────────────────────────────────
    let listen: String = Input::with_theme(&theme)
        .with_prompt("Gateway listen address")
        .default("0.0.0.0:8080".to_string())
        .interact_text()?;

    let p2p_listen: String = Input::with_theme(&theme)
        .with_prompt("P2P listen address")
        .default("/ip4/0.0.0.0/tcp/7051".to_string())
        .interact_text()?;

    let data_dir: String = Input::with_theme(&theme)
        .with_prompt("Data directory")
        .default("./data".to_string())
        .interact_text()?;

    // ── Step 5: CA passphrase ─────────────────────────────────────────────────
    let passphrase = Password::with_theme(&theme)
        .with_prompt("CA passphrase")
        .with_confirmation("Confirm passphrase", "Passphrases do not match")
        .interact()?;

    // ── Summary & confirm ─────────────────────────────────────────────────────
    println!();
    println!("  Summary");
    println!("  ───────────────────────────────────────────");
    println!("  Org      : {}", org);
    println!("  Admin    : {} [ADMIN]", admin_name);
    println!("  Role     : {}", node_role);
    println!("  Listen   : {}", listen);
    println!("  P2P      : {}", p2p_listen);
    println!("  Data dir : {}", data_dir);
    println!("  CA dir   : ~/.qorvum/ca/{}", org.to_lowercase());
    println!();

    if !Confirm::with_theme(&theme).with_prompt("Proceed?").default(true).interact()? {
        println!("Aborted.");
        return Ok(());
    }

    println!();

    // ── Execute ───────────────────────────────────────────────────────────────
    // 1. CA init
    print!("  Initializing CA...");
    cmd_ca_init(org.clone(), false, None, Some(passphrase.clone()))?;
    println!(" done");

    // 2. Issue admin cert
    print!("  Issuing admin certificate...");
    cmd_ca_issue(None, Some(org.clone()), admin_name.clone(), "ADMIN".to_string(), 365, false, None)?;
    println!(" done");

    // 3. Set active identity
    let id_dir = default_identities_dir();
    let cert_path = id_dir.join(format!("{}.cert", admin_name));
    let key_path  = id_dir.join(format!("{}.key",  admin_name));
    print!("  Setting active identity...");
    cmd_identity_use(cert_path, key_path)?;
    println!(" done");

    // 4. Write config/node.yml
    print!("  Writing config/node.yml...");
    let ca_dir_str = default_ca_dir(&org).display().to_string();
    let config_yaml = format!(
        "# Qorvum node configuration — generated by `qv init`\norg: {org}\n\nnode:\n  role: {node_role}\n  listen: \"{listen}\"\n  p2p_listen: \"{p2p_listen}\"\n  data_dir: \"{data_dir}\"\n  channel: main-channel\n  log_level: info\n\nca:\n  dir: \"{ca_dir_str}\"\n  # passphrase: set via QORVUM_CA_PASSPHRASE env var instead\n\npeers: []\nvalidator_keys: []\n"
    );
    std::fs::create_dir_all("config")?;
    std::fs::write("config/node.yml", &config_yaml)?;
    println!(" done");

    println!();
    println!("  Setup complete!");
    println!();
    println!("  Start your node:");
    println!("    cargo run -p qorvum-node");
    println!();
    println!("  Check health:");
    println!("    qv health");
    println!();
    println!("  Live dashboard:");
    println!("    qv node top");
    println!();

    Ok(())
}

// ── Node top TUI ──────────────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct NodeMetrics {
    block_height:  u64,
    uptime_secs:   u64,
    cpu_percent:   f64,
    mem_used_mb:   u64,
    mem_total_mb:  u64,
    mem_percent:   u64,
    disk_mb:       u64,
    recent_blocks: Vec<BlockSummary>,
    error:         Option<String>,
}

#[derive(Default, Clone)]
struct BlockSummary {
    number:    u64,
    tx_count:  u64,
    hash:      String,
    timestamp: u64,
}

async fn fetch_metrics(url: &str) -> NodeMetrics {
    let endpoint = format!("{}/api/v1/metrics", url.trim_end_matches('/'));
    match reqwest::get(&endpoint).await {
        Err(e) => NodeMetrics { error: Some(format!("Connection failed: {e}")), ..Default::default() },
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Err(e) => NodeMetrics { error: Some(format!("Parse error: {e}")), ..Default::default() },
            Ok(json) => {
                let d = &json["data"];
                let blocks = d["recent_blocks"].as_array()
                    .map(|arr| arr.iter().map(|b| BlockSummary {
                        number:   b["number"].as_u64().unwrap_or(0),
                        tx_count: b["tx_count"].as_u64().unwrap_or(0),
                        hash:     b["hash"].as_str().unwrap_or("").to_string(),
                        timestamp: b["timestamp"].as_u64().unwrap_or(0),
                    }).collect())
                    .unwrap_or_default();
                NodeMetrics {
                    block_height:  d["block_height"].as_u64().unwrap_or(0),
                    uptime_secs:   d["uptime_secs"].as_u64().unwrap_or(0),
                    cpu_percent:   d["cpu_percent"].as_f64().unwrap_or(0.0),
                    mem_used_mb:   d["mem_used_mb"].as_u64().unwrap_or(0),
                    mem_total_mb:  d["mem_total_mb"].as_u64().unwrap_or(0),
                    mem_percent:   d["mem_percent"].as_u64().unwrap_or(0),
                    disk_mb:       d["disk_mb"].as_u64().unwrap_or(0),
                    recent_blocks: blocks,
                    error:         None,
                }
            }
        }
    }
}

fn fmt_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 { format!("{h}h {m}m {s}s") }
    else if m > 0 { format!("{m}m {s}s") }
    else { format!("{s}s") }
}

fn fmt_ts(unix_secs: u64) -> String {
    Utc.timestamp_opt(unix_secs as i64, 0)
        .single()
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "?".to_string())
}

fn gauge_bar(pct: u64) -> Gauge<'static> {
    let color = if pct > 85 { Color::Red }
                else if pct > 60 { Color::Yellow }
                else { Color::Green };
    Gauge::default()
        .gauge_style(Style::default().fg(color))
        .percent(pct.min(100) as u16)
}

fn draw_top(f: &mut ratatui::Frame, metrics: &NodeMetrics, url: &str, tick: u64) {
    let area = f.area();

    // Root layout: header | body | footer
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // ── Header ────────────────────────────────────────────────────────────────
    let uptime_str  = fmt_uptime(metrics.uptime_secs);
    let header_text = if let Some(err) = &metrics.error {
        Line::from(vec![
            Span::styled(" ERROR ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" {err}")),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Qorvum Node ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!("  url: {}   uptime: {}   [q] quit", url, uptime_str)),
        ])
    };
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, root[0]);

    // ── Body: left (blockchain) | right (system) ─────────────────────────────
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);

    // Split left into stats | recent blocks
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(body[0]);

    // ── Blockchain stats ──────────────────────────────────────────────────────
    let block_text = vec![
        Line::from(vec![
            Span::raw("  Height    "),
            Span::styled(
                format!("{}", metrics.block_height),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(format!("  Disk      {} MB (RocksDB)", metrics.disk_mb)),
    ];
    let block_panel = Paragraph::new(block_text)
        .block(Block::default().borders(Borders::ALL).title(" Blockchain "));
    f.render_widget(block_panel, left[0]);

    // ── Recent blocks ─────────────────────────────────────────────────────────
    let block_items: Vec<ListItem> = metrics.recent_blocks.iter().map(|b| {
        let ts = fmt_ts(b.timestamp);
        ListItem::new(Line::from(vec![
            Span::styled(
                format!(" #{:<6}", b.number),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(format!("  {:>8}  {:>3} tx  {}", b.hash, b.tx_count, ts)),
        ]))
    }).collect();
    let block_list = List::new(block_items)
        .block(Block::default().borders(Borders::ALL).title(" Recent Blocks "));
    f.render_widget(block_list, left[1]);

    // ── System stats ──────────────────────────────────────────────────────────
    let sys_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(body[1]);

    // CPU panel
    let cpu_label = format!(" CPU  {:.1}%", metrics.cpu_percent);
    let cpu_block = Block::default().borders(Borders::ALL).title(" System ");
    let inner_sys = cpu_block.inner(sys_chunks[0]);
    f.render_widget(cpu_block, sys_chunks[0]);

    let cpu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner_sys);
    f.render_widget(Paragraph::new(cpu_label), cpu_layout[0]);
    f.render_widget(gauge_bar(metrics.cpu_percent as u64), cpu_layout[1]);

    // RAM panel
    let ram_label = format!(" RAM  {} / {} MB", metrics.mem_used_mb, metrics.mem_total_mb);
    let ram_inner = Block::default().borders(Borders::ALL).inner(sys_chunks[1]);
    f.render_widget(Block::default().borders(Borders::ALL), sys_chunks[1]);
    let ram_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(ram_inner);
    f.render_widget(Paragraph::new(ram_label), ram_layout[0]);
    f.render_widget(gauge_bar(metrics.mem_percent), ram_layout[1]);

    // Disk panel (text only)
    let disk_text = format!("\n  Disk  {} MB  (RocksDB ledger)", metrics.disk_mb);
    let disk_panel = Paragraph::new(disk_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(disk_panel, sys_chunks[2]);

    // ── Footer ────────────────────────────────────────────────────────────────
    let dot = if tick % 2 == 0 { "●" } else { "○" };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {dot} "), Style::default().fg(Color::Green)),
        Span::raw(format!("Refreshing every 2s   last update: {}", fmt_ts(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ))),
    ]));
    f.render_widget(footer, root[2]);
}

async fn cmd_node_top(url: String) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut metrics = NodeMetrics::default();
    let mut tick    = 0u64;
    let poll_every  = std::time::Duration::from_secs(2);
    let mut last_fetch = std::time::Instant::now()
        .checked_sub(poll_every)
        .unwrap_or_else(std::time::Instant::now);

    loop {
        // Fetch metrics every 2 s
        if last_fetch.elapsed() >= poll_every {
            metrics    = fetch_metrics(&url).await;
            last_fetch = std::time::Instant::now();
            tick      += 1;
        }

        term.draw(|f| draw_top(f, &metrics, &url, tick))?;

        // Non-blocking key check (100ms timeout)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    Ok(())
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
        // ── Init wizard ───────────────────────────────────────────────────────
        Commands::Init { org } => cmd_init(org)?,

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
            NodeCommands::Top => cmd_node_top(cli.url).await?,
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

        Commands::Contract { action } => match action {
            ContractCommands::Deploy { name, wasm } => {
                let bearer = get_bearer_token().context("ADMIN token required — run `qv identity token` first")?;
                let wasm_bytes = std::fs::read(&wasm)
                    .with_context(|| format!("Cannot read WASM file: {}", wasm.display()))?;

                if !wasm_bytes.starts_with(b"\0asm") {
                    bail!("File does not look like a WASM binary (magic bytes mismatch)");
                }

                let file_part = reqwest::multipart::Part::bytes(wasm_bytes)
                    .file_name(format!("{}.wasm", name))
                    .mime_str("application/wasm")?;
                let form = reqwest::multipart::Form::new()
                    .text("contract_id", name.clone())
                    .part("wasm", file_part);

                let client = reqwest::Client::new();
                let base   = cli.url.trim_end_matches('/');
                let resp = client
                    .post(format!("{}/api/v1/contracts/deploy", base))
                    .bearer_auth(&bearer)
                    .multipart(form)
                    .send().await.context("Request failed")?;

                let json: serde_json::Value = resp.json().await?;
                if json["success"].as_bool().unwrap_or(false) {
                    let d = &json["data"];
                    println!("Deployed contract '{}' ({} bytes) — status: {}",
                        d["contract_id"].as_str().unwrap_or(&name),
                        d["size_bytes"].as_u64().unwrap_or(0),
                        d["status"].as_str().unwrap_or("unknown"),
                    );
                } else {
                    bail!("Deploy failed: {}", json["error"]["message"].as_str().unwrap_or("unknown"));
                }
            }

            ContractCommands::List => {
                let bearer = get_bearer_token().ok();
                let client = reqwest::Client::new();
                let base   = cli.url.trim_end_matches('/');
                let req    = client.get(format!("{}/api/v1/contracts", base));
                let json   = send_api(&client, req, bearer).await?;

                let contracts = json["data"]["contracts"].as_array()
                    .cloned()
                    .unwrap_or_default();

                if contracts.is_empty() {
                    println!("No contracts deployed.");
                } else {
                    let mut table = Table::new();
                    table.load_preset(UTF8_FULL);
                    table.set_header(vec!["Contract ID", "Kind", "Functions"]);
                    for c in &contracts {
                        let fns = c["functions"].as_array()
                            .map(|a| a.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", "))
                            .unwrap_or_default();
                        table.add_row(vec![
                            c["id"].as_str().unwrap_or("-"),
                            c["kind"].as_str().unwrap_or("-"),
                            &fns,
                        ]);
                    }
                    println!("{table}");
                }
            }
        },
    }
    Ok(())
}