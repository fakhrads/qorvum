use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use qorvum_contracts::executor::ContractExecutor;
use qorvum_consensus::ConsensusEngine;
use qorvum_ledger::backends::RocksDbStore;
use qorvum_ledger::query::QueryEngine;
use qorvum_ledger::store::LedgerStore;
use qorvum_msp::{CertificateAuthority, IdentityVerifier, UserStore};

use crate::sse::EventBroadcaster;

pub struct AppState {
    pub channel_id:   String,
    pub store:        Arc<dyn LedgerStore>,
    pub executor:     RwLock<ContractExecutor>,
    pub query_engine: QueryEngine,
    pub consensus:    Option<Arc<ConsensusEngine>>,
    pub verifier:     Option<Arc<RwLock<IdentityVerifier>>>,
    pub ca:           Option<Arc<Mutex<CertificateAuthority>>>,
    pub user_store:   Option<Arc<UserStore>>,
    /// SSE broadcaster — push real-time events ke semua connected frontend
    pub broadcaster:  Arc<EventBroadcaster>,
    /// Node data directory path, used for disk usage metrics
    pub data_dir:     String,
    /// Process start time for uptime calculation
    pub start_time:   std::time::Instant,
}

impl AppState {
    pub fn new(channel_id: &str, data_dir: &str) -> Self {
        let store = Self::open_store(data_dir);
        Self::with_all(channel_id, data_dir.to_string(), store, None, None)
    }

    pub fn new_with_store(channel_id: &str, data_dir: &str, store: Arc<dyn LedgerStore>) -> Self {
        Self::with_all(channel_id, data_dir.to_string(), store, None, None)
    }

    pub fn new_with_consensus(
        channel_id: &str,
        data_dir:   &str,
        store:      Arc<dyn LedgerStore>,
        consensus:  Arc<ConsensusEngine>,
    ) -> Self {
        Self::with_all(channel_id, data_dir.to_string(), store, Some(consensus), None)
    }

    pub fn new_with_consensus_and_pki(
        channel_id: &str,
        data_dir:   &str,
        store:      Arc<dyn LedgerStore>,
        consensus:  Arc<ConsensusEngine>,
        verifier:   Arc<RwLock<IdentityVerifier>>,
    ) -> Self {
        Self::with_all(channel_id, data_dir.to_string(), store, Some(consensus), Some(verifier))
    }

    pub fn set_verifier(&mut self, verifier: Arc<RwLock<IdentityVerifier>>) {
        self.verifier = Some(verifier);
    }

    pub fn set_consensus(&mut self, consensus: Arc<ConsensusEngine>) {
        self.consensus = Some(consensus);
    }

    pub fn load_verifier(ca_dir: &Path) -> Option<Arc<RwLock<IdentityVerifier>>> {
        if !ca_dir.exists() || !ca_dir.join("ca.cert").exists() {
            warn!(
                "No CA configured at {:?} — DEVELOPMENT mode (all identities accepted)",
                ca_dir
            );
            return None;
        }
        match IdentityVerifier::new(&[ca_dir.to_path_buf()]) {
            Ok(v) if v.is_configured() => {
                info!("PKI loaded from {:?} — token verification enabled", ca_dir);
                Some(Arc::new(RwLock::new(v)))
            }
            Ok(_) => {
                warn!("CA directory exists but no valid CA — DEVELOPMENT mode");
                None
            }
            Err(e) => {
                warn!("Failed to load CA: {} — DEVELOPMENT mode", e);
                None
            }
        }
    }

    pub fn enable_enrollment(&mut self, ca_dir: &Path, passphrase: &str) {
        match CertificateAuthority::load(ca_dir, passphrase) {
            Ok(ca) => {
                self.user_store = Some(Arc::new(UserStore::new(ca_dir)));
                self.ca = Some(Arc::new(Mutex::new(ca)));
                info!("CA enrollment enabled — admin endpoints active");
            }
            Err(e) => {
                warn!("Failed to load CA: {} — enrollment disabled", e);
            }
        }
    }

    fn open_store(data_dir: &str) -> Arc<dyn LedgerStore> {
        let db_path = Path::new(data_dir).join("ledger");
        info!("Opening RocksDB at {:?}", db_path);
        Arc::new(RocksDbStore::open(&db_path).expect("Failed to open RocksDB"))
    }

    fn with_all(
        channel_id: &str,
        data_dir:   String,
        store:      Arc<dyn LedgerStore>,
        consensus:  Option<Arc<ConsensusEngine>>,
        verifier:   Option<Arc<RwLock<IdentityVerifier>>>,
    ) -> Self {
        let executor = {
            let mut e = ContractExecutor::new(store.clone());
            if !data_dir.is_empty() {
                e = e.with_persistence(&data_dir);
                e.load_persisted();
            }
            e
        };
        let query_engine = QueryEngine::new(store.clone());
        let broadcaster  = Arc::new(EventBroadcaster::new());

        crate::sse::spawn_heartbeat(broadcaster.clone());

        Self {
            channel_id: channel_id.to_string(),
            store,
            executor:    RwLock::new(executor),
            query_engine,
            consensus,
            verifier,
            ca:          None,
            user_store:  None,
            broadcaster,
            data_dir,
            start_time:  std::time::Instant::now(),
        }
    }
}