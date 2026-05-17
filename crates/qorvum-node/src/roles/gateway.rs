//! GatewayRole — serves the REST API.
//!
//! When a ConsensusEngine handle is provided (via `with_consensus`), transaction
//! execution goes through consensus before committing. Otherwise, the gateway
//! falls back to direct single-node commit (dev/standalone mode).

use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Scan `./contracts/` for pre-built WASM release artifacts and hot-load them.
/// Convention: `contracts/<name>/build/release.wasm` → contract_id = `<name>`
async fn load_wasm_contracts(executor: &mut qorvum_contracts::ContractExecutor) {
    let contracts_dir = PathBuf::from("contracts");
    if !contracts_dir.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&contracts_dir) {
        Ok(e) => e,
        Err(e) => { warn!("[gateway] Cannot scan contracts/: {}", e); return; }
    };
    for entry in entries.flatten() {
        let wasm_path = entry.path().join("build").join("release.wasm");
        if !wasm_path.is_file() {
            continue;
        }
        let contract_id = entry.file_name().to_string_lossy().to_string();
        match std::fs::read(&wasm_path) {
            Ok(bytes) => {
                info!("[gateway] Auto-loaded WASM contract '{}' ({} bytes)", contract_id, bytes.len());
                executor.deploy_wasm(&contract_id, bytes);
            }
            Err(e) => warn!("[gateway] Failed to load WASM '{}': {}", contract_id, e),
        }
    }
}

use qorvum_consensus::ConsensusEngine;
use qorvum_ledger::store::LedgerStore;

use crate::bus::NodeBus;

pub struct GatewayRole {
    listen:         String,
    channel:        String,
    data_dir:       String,
    ca_dir:         PathBuf,
    ca_passphrase:  Option<String>,
    store:          Arc<dyn LedgerStore>,
    /// Injected when the node also runs a ConsensusRole (--role all or combined roles).
    consensus:      Option<Arc<ConsensusEngine>>,
    #[allow(dead_code)]
    bus:            NodeBus,
}

impl GatewayRole {
    pub fn new(
        listen:        String,
        channel:       String,
        data_dir:      String,
        ca_dir:        PathBuf,
        ca_passphrase: Option<String>,
        store:         Arc<dyn LedgerStore>,
        bus:           NodeBus,
    ) -> Self {
        Self {
            listen, channel, data_dir, ca_dir, ca_passphrase, store,
            consensus: None,
            bus,
        }
    }

    /// Call this when a ConsensusEngine is available in the same process.
    pub fn with_consensus(mut self, consensus: Arc<ConsensusEngine>) -> Self {
        self.consensus = Some(consensus);
        self
    }

    pub async fn run(self) {
        info!("[gateway] Role started — listening on {}", self.listen);

        let verifier = qorvum_gateway::AppState::load_verifier(&self.ca_dir);
        let mode = if self.consensus.is_some() { "consensus" } else { "dev" };

        let mut app_state = qorvum_gateway::AppState::new_with_store(
            &self.channel,
            &self.data_dir,
            self.store.clone(),
        );

        if let Some(consensus) = self.consensus {
            app_state.set_consensus(consensus);
        } else {
            warn!(
                "[gateway] No local ConsensusEngine — transactions will be committed directly \
                (single-node mode). For production clusters run with --role validator,gateway"
            );
        }

        if let Some(passphrase) = &self.ca_passphrase {
            app_state.enable_enrollment(&self.ca_dir, passphrase);
        }

        if let Some(v) = verifier {
            app_state.set_verifier(v);
        }

        {
            let mut executor = app_state.executor.write().await;
            executor.register_native("hr-service", hr_service::register());
            load_wasm_contracts(&mut executor).await;
        }

        // Wire peer topology updates → broadcaster node_status events
        let broadcaster = app_state.broadcaster.clone();
        let store_for_status = self.store.clone();
        let mut peer_rx = self.bus.peer_status_rx();
        tokio::spawn(async move {
            while peer_rx.changed().await.is_ok() {
                let peers: Vec<qorvum_gateway::sse::PeerInfo> = peer_rx.borrow()
                    .iter()
                    .map(|p| qorvum_gateway::sse::PeerInfo {
                        peer_id: p.peer_id.clone(),
                        addr:    p.addr.clone(),
                    })
                    .collect();
                let latest = store_for_status.get_latest_block_num().unwrap_or(None);
                broadcaster.node_status("ok", peers.len(), latest, mode, peers);
            }
        });

        let app = qorvum_gateway::build_router(Arc::new(app_state));
        let addr: std::net::SocketAddr = self.listen.parse()
            .expect("[gateway] Invalid listen address");

        let listener = tokio::net::TcpListener::bind(&addr).await
            .expect("[gateway] Failed to bind");

        info!("[gateway] REST API ready at http://{}", addr);
        axum::serve(listener, app).await
            .expect("[gateway] Server exited unexpectedly");
    }
}