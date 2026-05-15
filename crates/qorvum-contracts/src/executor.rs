//! ContractExecutor — runs native Rust and WASM (AssemblyScript/any) contracts.

use crate::context::ChainContextImpl;
use crate::wasm_host;
use chain_sdk::ChainContext;
use qorvum_ledger::store::LedgerStore;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

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
    wasm_registry:    HashMap<String, Vec<u8>>,  // contract_id → wasm bytes
}

impl ContractExecutor {
    pub fn new(store: Arc<dyn LedgerStore>) -> Self {
        Self {
            store,
            native_registry: HashMap::new(),
            wasm_registry:   HashMap::new(),
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

    /// Deploy WASM contract bytes
    pub fn deploy_wasm(&mut self, contract_id: &str, wasm_bytes: Vec<u8>) {
        info!("Deployed WASM contract: {} ({} bytes)", contract_id, wasm_bytes.len());
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
