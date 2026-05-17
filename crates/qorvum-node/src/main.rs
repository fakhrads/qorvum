//! Qorvum Peer Node — Role-Based Architecture
//!
//! Each node can run as one or more roles:
//!   --role validator  → participates in HotStuff BFT consensus
//!   --role gateway    → serves REST API to clients
//!   --role peer       → syncs ledger and relays transactions (no consensus voting)
//!   --role all        → all of the above (default, single-node dev mode)
//!
//! Roles communicate through typed tokio channels (NodeBus), sharing only
//! a single Arc<dyn LedgerStore> for storage access.

mod bus;
mod roles;

use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use bus::NodeBus;
use roles::{
    consensus::ConsensusRole,
    gateway::GatewayRole,
    peer::PeerRole,
};

use qorvum_crypto::signing::{PQKeypair, PublicKey, SigningAlgorithm};
use qorvum_ledger::{backends::RocksDbStore, store::LedgerStore};

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum NodeRole {
    /// Participates in HotStuff BFT consensus, proposes and votes on blocks
    Validator,
    /// Serves the REST API, authenticates clients, forwards transactions
    Gateway,
    /// Syncs ledger from peers, relays client transactions but does not vote
    Peer,
    /// Runs all roles in one process (default for single-node / dev)
    All,
}

impl std::fmt::Display for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRole::Validator => write!(f, "validator"),
            NodeRole::Gateway   => write!(f, "gateway"),
            NodeRole::Peer      => write!(f, "peer"),
            NodeRole::All       => write!(f, "all"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "qorvum-node",
    about = "Qorvum Enterprise Post-Quantum Blockchain Node",
    long_about = "Start a node with one or more roles.\n\
        Use --role all for single-node dev/staging.\n\
        Use --role validator/gateway/peer for production cluster deployment.",
)]
pub struct Cli {
    /// Role(s) this node will perform.
    /// Can be specified multiple times: --role validator --role gateway
    #[arg(
        long,
        value_enum,
        default_value = "all",
        env = "QORVUM_ROLE",
    )]
    pub role: Vec<NodeRole>,

    #[arg(long, default_value = "0.0.0.0:8080", env = "QORVUM_LISTEN")]
    pub listen: String,

    #[arg(long, default_value = "main-channel", env = "QORVUM_CHANNEL")]
    pub channel: String,

    #[arg(long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,

    #[arg(long, default_value = "/ip4/0.0.0.0/tcp/7051", env = "QORVUM_P2P_LISTEN")]
    pub p2p_listen: String,

    #[arg(long, default_value = "./data/node1", env = "QORVUM_DATA_DIR")]
    pub data_dir: String,

    /// Directory containing `ca.cert` and `crl.json` for PKI token verification.
    /// If absent, gateway runs in DEVELOPMENT mode (accepts X-Identity headers).
    #[arg(long, default_value = "./ca", env = "QORVUM_CA_DIR")]
    pub ca_dir: PathBuf,

    /// Passphrase for the CA private key.
    /// When set, enables REST enrollment/revocation endpoints.
    #[arg(long, env = "QORVUM_CA_PASSPHRASE")]
    pub ca_passphrase: Option<String>,

    /// Hex-encoded Dilithium3 public keys of other validators (comma-separated).
    /// This node's own key is always included when running as validator.
    #[arg(long, env = "QORVUM_VALIDATOR_KEYS", value_delimiter = ',')]
    pub validator_keys: Vec<String>,

    /// Address of an existing node to connect to for initial sync.
    /// Format: /ip4/10.0.0.1/tcp/7051/p2p/<PEER_ID>
    /// Can be specified multiple times for multiple bootstrap peers.
    #[arg(long, env = "QORVUM_BOOTSTRAP_PEERS", value_delimiter = ',')]
    pub bootstrap_peers: Vec<String>,

    /// Path to a YAML config file. Defaults to config/node.yml or config/node.yaml.
    #[arg(long, env = "QORVUM_CONFIG")]
    pub config: Option<PathBuf>,
}

impl Cli {
    pub fn has_role(&self, role: &NodeRole) -> bool {
        self.role.contains(&NodeRole::All) || self.role.contains(role)
    }

    pub fn active_roles(&self) -> Vec<&NodeRole> {
        if self.role.contains(&NodeRole::All) {
            vec![&NodeRole::Validator, &NodeRole::Gateway, &NodeRole::Peer]
        } else {
            self.role.iter().collect()
        }
    }
}

// ── config/node.yml ───────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
struct NodeConfig {
    #[serde(default)]
    node: NodeSection,
    #[serde(default)]
    ca: CaSection,
    #[serde(default)]
    peers: Vec<String>,
    #[serde(default)]
    validator_keys: Vec<String>,
}

#[derive(serde::Deserialize, Default)]
struct NodeSection {
    role:       Option<String>,
    listen:     Option<String>,
    p2p_listen: Option<String>,
    data_dir:   Option<String>,
    channel:    Option<String>,
    log_level:  Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct CaSection {
    dir:        Option<String>,
    passphrase: Option<String>,
}

fn load_node_config(explicit: Option<&std::path::Path>) -> NodeConfig {
    let candidates: Vec<&std::path::Path> = if let Some(p) = explicit {
        vec![p]
    } else {
        vec![
            std::path::Path::new("config/node.yml"),
            std::path::Path::new("config/node.yaml"),
        ]
    };

    for path in candidates {
        if path.exists() {
            return match std::fs::read_to_string(path) {
                Ok(s) => serde_yaml::from_str(&s).unwrap_or_else(|e| {
                    eprintln!("[warn] {} parse error: {e} — using defaults", path.display());
                    NodeConfig::default()
                }),
                Err(e) => {
                    eprintln!("[warn] cannot read {}: {e}", path.display());
                    NodeConfig::default()
                }
            };
        } else if explicit.is_some() {
            eprintln!("[warn] config file not found: {}", path.display());
        }
    }
    NodeConfig::default()
}

/// Apply config/node.yml values where CLI still holds the default (env not set).
/// CLI args always win; config file fills in what's missing.
fn apply_config(cli: &mut Cli, cfg: NodeConfig) {
    let env_set = |var: &str| std::env::var(var).is_ok();

    if !env_set("QORVUM_ROLE") && cli.role == vec![NodeRole::All] {
        if let Some(r) = &cfg.node.role {
            if let Ok(parsed) = r.parse::<NodeRole>() {
                cli.role = vec![parsed];
            }
        }
    }
    macro_rules! fill {
        ($field:expr, $val:expr, $env:literal) => {
            if !env_set($env) {
                if let Some(v) = $val { $field = v; }
            }
        };
    }
    fill!(cli.listen,     cfg.node.listen,     "QORVUM_LISTEN");
    fill!(cli.p2p_listen, cfg.node.p2p_listen, "QORVUM_P2P_LISTEN");
    fill!(cli.data_dir,   cfg.node.data_dir,   "QORVUM_DATA_DIR");
    fill!(cli.channel,    cfg.node.channel,    "QORVUM_CHANNEL");
    fill!(cli.log_level,  cfg.node.log_level,  "RUST_LOG");

    if !env_set("QORVUM_CA_DIR") {
        if let Some(d) = cfg.ca.dir { cli.ca_dir = d.into(); }
    }
    if !env_set("QORVUM_CA_PASSPHRASE") && cli.ca_passphrase.is_none() {
        cli.ca_passphrase = cfg.ca.passphrase;
    }
    if !env_set("QORVUM_BOOTSTRAP_PEERS") && cli.bootstrap_peers.is_empty() {
        cli.bootstrap_peers = cfg.peers;
    }
    if !env_set("QORVUM_VALIDATOR_KEYS") && cli.validator_keys.is_empty() {
        cli.validator_keys = cfg.validator_keys.into_iter()
            .filter_map(|v| resolve_validator_key(&v))
            .collect();
    }
}

/// Jika value adalah path file yang ada, baca dan extract hex pubkey dari validator.key.
/// Jika bukan path (hex string biasa), kembalikan as-is.
fn resolve_validator_key(val: &str) -> Option<String> {
    let path = std::path::Path::new(val);
    // Anggap path jika mengandung separator atau ekstensi .key
    let looks_like_path = val.contains('/') || val.contains('\\') || val.ends_with(".key");

    if looks_like_path {
        // Coba baca sebagai file; kalau tidak ada, coba tambahkan /validator.key
        let key_path = if path.is_file() {
            path.to_path_buf()
        } else {
            path.join("validator.key")
        };

        match std::fs::read(&key_path) {
            Ok(bytes) => {
                match bincode::deserialize::<(u8, Vec<u8>, Vec<u8>)>(&bytes) {
                    Ok((_alg, pk_bytes, _sk)) => Some(hex::encode(&pk_bytes)),
                    Err(e) => {
                        eprintln!("[warn] validator key file {:?} corrupt: {e}", key_path);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("[warn] cannot read validator key {:?}: {e}", key_path);
                None
            }
        }
    } else {
        Some(val.to_string())
    }
}

impl std::str::FromStr for NodeRole {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "validator" => Ok(NodeRole::Validator),
            "gateway"   => Ok(NodeRole::Gateway),
            "peer"      => Ok(NodeRole::Peer),
            "all"       => Ok(NodeRole::All),
            other       => Err(format!("unknown role: {other}")),
        }
    }
}

// ── Entry Point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let mut cli  = Cli::parse();
    let node_cfg = load_node_config(cli.config.as_deref());
    apply_config(&mut cli, node_cfg);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cli.log_level))
        .with_target(false)
        .compact()
        .init();

    print_banner(&cli);

    // Validate role combinations
    validate_roles(&cli)?;

    // ── Shared: persistent storage ────────────────────────────────────────────
    let store = open_store(&cli.data_dir)?;

    // ── Shared: validator keypair (always generated; used only when validator) ─
    let key_path = std::path::Path::new(&cli.data_dir).join("validator.key");
    let keypair = load_or_generate_keypair(&key_path);

    if cli.has_role(&NodeRole::Validator) {
        info!("Validator pubkey: {}", keypair.public_key().to_hex());
    }

    // ── Shared: internal message bus ─────────────────────────────────────────
    let bus = NodeBus::new();

    // ── Build ConsensusEngine (shared between ConsensusRole + GatewayRole) ──────
    //
    // When running --role all or --role validator,gateway the ConsensusEngine is
    // constructed once and shared via Arc so the gateway can call propose_block()
    // directly without going through a channel round-trip.
    let consensus_engine: Option<Arc<qorvum_consensus::ConsensusEngine>> =
        if cli.has_role(&NodeRole::Validator) {
            let validator_keys = build_validator_keys(&cli, &keypair);
            let quorum = qorvum_consensus::ValidatorSet::new(validator_keys.clone()).quorum_size();
            info!(
                "Validator set: {} node(s), quorum = {}",
                validator_keys.len(),
                quorum
            );

            // ConsensusEngine → PeerRole forwarding channel (wired in PeerRole via bus)
            let net_tx = bus.p2p_out_tx.clone();
            // Wrap into Vec<u8> sender that ConsensusEngine expects
            let (raw_tx, mut raw_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(512);
            let p2p_topic_tx = net_tx.clone();
            tokio::spawn(async move {
                while let Some(data) = raw_rx.recv().await {
                    let _ = p2p_topic_tx.send(crate::bus::P2POutbound {
                        topic: "qorvum-consensus".to_string(),
                        data,
                    }).await;
                }
            });

            Some(qorvum_consensus::ConsensusEngine::new(
                qorvum_consensus::ValidatorSet::new(validator_keys),
                keypair,
                store.clone(),
                raw_tx,
            ))
        } else {
            None
        };

    // ── Spawn roles as independent async tasks ────────────────────────────────
    let mut handles = Vec::new();

    // Consensus role: processes TxSubmissions from bus, drives propose_block
    if let Some(ref engine) = consensus_engine {
        let role = ConsensusRole::new(
            engine.clone(),
            store.clone(),
            bus.clone(),
        );
        handles.push(tokio::spawn(async move {
            role.run().await
        }));
    }

    // Peer / networking role (needed by both validator and pure peer)
    if cli.has_role(&NodeRole::Validator) || cli.has_role(&NodeRole::Peer) {
        let role = PeerRole::new(
            cli.p2p_listen.clone(),
            cli.bootstrap_peers.clone(),
            store.clone(),
            bus.clone(),
        );
        handles.push(tokio::spawn(async move {
            role.run().await
        }));
    }

    // Gateway role — inject ConsensusEngine if running in same process
    if cli.has_role(&NodeRole::Gateway) {
        let gateway_role = GatewayRole::new(
            cli.listen.clone(),
            cli.channel.clone(),
            cli.data_dir.clone(),
            cli.ca_dir.clone(),
            cli.ca_passphrase.clone(),
            store.clone(),
            bus.clone(),
        );
        let gateway_role = if let Some(engine) = &consensus_engine {
            gateway_role.with_consensus(engine.clone())
        } else {
            gateway_role
        };
        handles.push(tokio::spawn(async move {
            gateway_role.run().await
        }));
    }

    info!("All roles started — node is operational");

    // Wait for any task to exit (they shouldn't in normal operation)
    let (result, idx, remaining) = futures::future::select_all(handles).await;
    tracing::error!("Role task #{} exited unexpectedly: {:?}", idx, result);
    drop(remaining);

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn print_banner(cli: &Cli) {
    info!("╔═══════════════════════════════════════════════════╗");
    info!("║   Qorvum Enterprise Post-Quantum Blockchain       ║");
    info!("║   Phase 5 — Role-Based Node Architecture          ║");
    info!("╚═══════════════════════════════════════════════════╝");
    info!("Channel  : {}", cli.channel);
    info!("Data Dir : {}", cli.data_dir);
    info!("CA Dir   : {:?}", cli.ca_dir);
    info!("Roles    : {}", cli.active_roles().iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", "));
    if cli.has_role(&NodeRole::Gateway) {
        info!("Listen   : {}", cli.listen);
    }
    if cli.has_role(&NodeRole::Validator) || cli.has_role(&NodeRole::Peer) {
        info!("P2P      : {}", cli.p2p_listen);
    }
    if !cli.bootstrap_peers.is_empty() {
        for peer in &cli.bootstrap_peers {
            info!("Bootstrap: {}", peer);
        }
    }
}

fn validate_roles(cli: &Cli) -> Result<()> {
    if cli.role.is_empty() {
        bail!("At least one --role must be specified");
    }

    // A gateway-only node can't commit blocks — it needs a validator or consensus
    // connection. Warn but allow (useful when gateway talks to remote validators).
    if cli.role == vec![NodeRole::Gateway] {
        tracing::warn!(
            "Running as gateway-only: transactions will be forwarded but no local consensus. \
            Ensure remote validators are reachable via P2P."
        );
    }

    Ok(())
}

fn open_store(data_dir: &str) -> Result<Arc<dyn LedgerStore>> {
    let db_path = std::path::Path::new(data_dir).join("ledger");
    std::fs::create_dir_all(&db_path)?;
    info!("Opening RocksDB at {:?}", db_path);
    let store = RocksDbStore::open(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to open RocksDB: {}", e))?;
    Ok(Arc::new(store))
}

fn build_validator_keys(cli: &Cli, own_keypair: &PQKeypair) -> Vec<PublicKey> {
    let mut keys = vec![own_keypair.public_key().clone()];
    for hex_key in &cli.validator_keys {
        let hex_key = hex_key.trim();
        if hex_key.is_empty() { continue; }
        match hex::decode(hex_key) {
            Ok(bytes) => keys.push(PublicKey {
                algorithm: SigningAlgorithm::Dilithium3,
                bytes,
            }),
            Err(e) => tracing::warn!("Ignoring invalid validator key '{hex_key}': {e}"),
        }
    }
    keys
}

fn load_or_generate_keypair(path: &std::path::Path) -> PQKeypair {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    if path.exists() {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok((alg_byte, pk_bytes, sk_bytes)) =
                bincode::deserialize::<(u8, Vec<u8>, Vec<u8>)>(&bytes)
            {
                let algorithm = if alg_byte == 0 {
                    SigningAlgorithm::Dilithium3
                } else {
                    SigningAlgorithm::Falcon512
                };
                info!("Loaded validator keypair from {:?}", path);
                return PQKeypair::from_bytes(algorithm, pk_bytes, sk_bytes);
            }
        }
        tracing::warn!("Keypair file corrupt or unreadable — regenerating");
    }

    info!("Generating new Dilithium3 validator keypair → {:?}", path);
    let kp = PQKeypair::generate(SigningAlgorithm::Dilithium3)
        .expect("Keypair generation failed");

    let alg_byte: u8 = 0;
    if let Ok(bytes) = bincode::serialize(&(
        alg_byte,
        kp.public_key().bytes.clone(),
        kp.secret_bytes(),
    )) {
        let _ = std::fs::write(path, &bytes);
    }

    kp
}