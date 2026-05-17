//! ContractExecutor — runs native Rust and WASM (AssemblyScript/any) contracts.

use crate::context::ChainContextImpl;
use crate::wasm_host;
use chain_sdk::ChainContext;
use qorvum_ledger::store::LedgerStore;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, debug, warn};

/// A native Rust contract function (for dev/testing without WASM)
pub type NativeFn = fn(&str, serde_json::Value, &dyn ChainContext) -> Result<serde_json::Value, String>;

/// Registered native contract
struct NativeContract {
    id:        String,
    functions: HashMap<String, NativeFn>,
}

/// Execution result from a contract call
#[derive(Debug)]
pub struct ExecResult {
    pub response:  serde_json::Value,
    pub events:    Vec<(String, Vec<u8>)>,
    pub ops:       Vec<qorvum_ledger::store::RecordOp>,
    pub reads:     Vec<(String, u64)>,
}

pub struct ContractExecutor {
    store:            Arc<dyn LedgerStore>,
    native_registry:  HashMap<String, NativeContract>,
    wasm_registry:    HashMap<String, Vec<u8>>,
    /// Directory where deployed WASM bytes are persisted: {data_dir}/contracts/
    contracts_dir:    Option<PathBuf>,
}

impl ContractExecutor {
    pub fn new(store: Arc<dyn LedgerStore>) -> Self {
        Self {
            store,
            native_registry: HashMap::new(),
            wasm_registry:   HashMap::new(),
            contracts_dir:   None,
        }
    }

    /// Enable persistence: deployed contracts survive node restarts.
    /// Call this before deploy_wasm / load_persisted.
    pub fn with_persistence(mut self, data_dir: &str) -> Self {
        let dir = PathBuf::from(data_dir).join("contracts");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            warn!("Cannot create contracts dir {:?}: {e}", dir);
        } else {
            self.contracts_dir = Some(dir);
        }
        self
    }

    /// Load all previously deployed contracts from disk.
    /// Call once on startup after with_persistence().
    pub fn load_persisted(&mut self) {
        let dir = match &self.contracts_dir {
            Some(d) => d.clone(),
            None    => return,
        };
        let entries = match std::fs::read_dir(&dir) {
            Ok(e)  => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("wasm") {
                continue;
            }
            let contract_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None    => continue,
            };
            match std::fs::read(&path) {
                Ok(bytes) => {
                    info!("Loaded persisted WASM contract '{}' ({} bytes)", contract_id, bytes.len());
                    self.wasm_registry.insert(contract_id, bytes);
                }
                Err(e) => warn!("Failed to load persisted contract {:?}: {e}", path),
            }
        }
    }

    /// Register a native Rust contract (dev mode — no WASM needed)
    pub fn register_native(
        &mut self,
        contract_id: &str,
        functions:   HashMap<String, NativeFn>,
    ) {
        info!("Registered native contract: {}", contract_id);
        self.native_registry.insert(contract_id.to_string(), NativeContract {
            id:        contract_id.to_string(),
            functions,
        });
    }

    /// Deploy WASM contract bytes and persist to disk if persistence is enabled.
    pub fn deploy_wasm(&mut self, contract_id: &str, wasm_bytes: Vec<u8>) {
        if let Some(dir) = &self.contracts_dir {
            let path = dir.join(format!("{}.wasm", contract_id));
            match std::fs::write(&path, &wasm_bytes) {
                Ok(_)  => info!("Deployed WASM contract '{}' ({} bytes) → {:?}", contract_id, wasm_bytes.len(), path),
                Err(e) => warn!("Deployed '{}' in-memory only — failed to persist: {e}", contract_id),
            }
        } else {
            info!("Deployed WASM contract '{}' ({} bytes) [in-memory only]", contract_id, wasm_bytes.len());
        }
        self.wasm_registry.insert(contract_id.to_string(), wasm_bytes);
    }

    /// Execute a contract function and return results.
    /// `verified` = true when the caller presented a cryptographically verified PQ certificate.
    pub fn execute(
        &self,
        contract_id:   &str,
        function_name: &str,
        args:          serde_json::Value,
        caller_id:     &str,
        caller_msp:    &str,
        caller_roles:  Vec<String>,
        tx_id:         [u8; 32],
        timestamp:     u64,
        verified:      bool,
    ) -> Result<ExecResult, String> {
        debug!("execute: {}/{} caller={} verified={}", contract_id, function_name, caller_id, verified);

        let ctx = Arc::new(ChainContextImpl::new_with_verified(
            self.store.clone(),
            caller_id.to_string(),
            caller_msp.to_string(),
            caller_roles,
            tx_id,
            timestamp,
            verified,
        ));

        // Try native first
        if let Some(cc) = self.native_registry.get(contract_id) {
            if let Some(func) = cc.functions.get(function_name) {
                let response = func(function_name, args, ctx.as_ref())
                    .map_err(|e| format!("Contract error: {}", e))?;

                return Ok(ExecResult {
                    response,
                    events: ctx.sim.drain_events(),
                    ops:    ctx.sim.drain_ops(),
                    reads:  ctx.sim.drain_reads(),
                });
            }
            return Err(format!("Function '{}' not found in contract '{}'", function_name, contract_id));
        }

        // Try WASM (AssemblyScript or any WASM target)
        if let Some(wasm_bytes) = self.wasm_registry.get(contract_id) {
            let wasm_bytes = wasm_bytes.clone();
            return wasm_host::execute_wasm(&wasm_bytes, function_name, args, ctx);
        }

        Err(format!("Contract '{}' not found", contract_id))
    }

    /// Returns all registered contract IDs with their type and function list.
    pub fn list_contracts(&self) -> Vec<serde_json::Value> {
        use serde_json::json;
        let mut result = Vec::new();
        for (id, cc) in &self.native_registry {
            let mut fns: Vec<&str> = cc.functions.keys().map(|s| s.as_str()).collect();
            fns.sort();
            result.push(json!({ "id": id, "kind": "native", "functions": fns }));
        }
        for id in self.wasm_registry.keys() {
            result.push(json!({ "id": id, "kind": "wasm", "functions": [] }));
        }
        result.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        result
    }

}
