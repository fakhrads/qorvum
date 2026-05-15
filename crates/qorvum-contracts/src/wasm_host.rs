//! WASM executor — runs AssemblyScript (or any WASM) contracts via wasmtime.
//!
//! # ABI contract
//! The WASM module must export:
//!   - `alloc(size: i32) -> i32`   raw heap allocator, no GC header
//!   - `dispatch(fn_ptr: i32, fn_len: i32, args_ptr: i32, args_len: i32) -> i64`
//!
//! Return encoding for i64: `(ptr << 32) | len`.
//! The host and the contract both use the same envelope JSON:
//!   `{"ok":true,"data":<value>}` on success
//!   `{"ok":false,"error":"<message>"}` on failure
//!
//! # Host imports (module "qv")
//! All ledger functions receive a JSON request string (ptr+len) and return an
//! i64-encoded pointer+length of a JSON envelope written into WASM memory.

use std::collections::HashMap;
use std::sync::Arc;

use chain_sdk::types::{FieldValue, Filter};
use chain_sdk::{ChainContext, Pagination, SortBy};
use serde_json::Value;
use wasmtime::{Caller, Engine, Linker, Module, Store};

use crate::context::ChainContextImpl;
use crate::executor::ExecResult;

struct WasmState {
    ctx: Arc<ChainContextImpl>,
}

// ── Memory helpers ────────────────────────────────────────────────────────────

fn read_bytes(caller: &mut Caller<'_, WasmState>, ptr: i32, len: i32) -> Result<Vec<u8>, String> {
    let mem = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or("WASM module must export 'memory'")?;
    let data = mem.data(&*caller);
    data.get(ptr as usize..(ptr + len) as usize)
        .map(|s| s.to_vec())
        .ok_or_else(|| format!("memory read out of bounds: ptr={} len={}", ptr, len))
}

fn read_str(caller: &mut Caller<'_, WasmState>, ptr: i32, len: i32) -> Result<String, String> {
    let bytes = read_bytes(caller, ptr, len)?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

fn write_to_wasm(caller: &mut Caller<'_, WasmState>, bytes: &[u8]) -> i64 {
    let len = bytes.len() as i32;

    let alloc_fn = match caller.get_export("alloc").and_then(|e| e.into_func()) {
        Some(f) => f,
        None => return encode_err_sentinel(),
    };
    let alloc_typed = match alloc_fn.typed::<i32, i32>(&mut *caller) {
        Ok(f) => f,
        Err(_) => return encode_err_sentinel(),
    };
    let ptr = match alloc_typed.call(&mut *caller, len) {
        Ok(p) => p,
        Err(_) => return encode_err_sentinel(),
    };

    let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return encode_err_sentinel(),
    };
    if mem.write(caller, ptr as usize, bytes).is_err() {
        return encode_err_sentinel();
    }

    ((ptr as i64) << 32) | (len as i64 & 0xFFFF_FFFF)
}

fn encode_err_sentinel() -> i64 { -1 }

fn respond(caller: &mut Caller<'_, WasmState>, result: Result<Value, String>) -> i64 {
    let envelope = match result {
        Ok(v)  => serde_json::json!({"ok": true,  "data":  v}),
        Err(e) => serde_json::json!({"ok": false, "error": e}),
    };
    let s = serde_json::to_vec(&envelope)
        .unwrap_or_else(|_| br#"{"ok":false,"error":"serialization failed"}"#.to_vec());
    write_to_wasm(caller, &s)
}

fn parse_fields(v: &Value) -> Result<HashMap<String, FieldValue>, String> {
    if v.is_null() || v.is_object() && v.as_object().unwrap().is_empty() {
        return Ok(HashMap::new());
    }
    serde_json::from_value(v.clone())
        .map_err(|e| format!("invalid fields format: {}", e))
}

// ── Host function registration ────────────────────────────────────────────────

fn register(linker: &mut Linker<WasmState>) -> anyhow::Result<()> {
    // qv_get {"collection","partition","id"}
    linker.func_wrap("qv", "qv_get", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col  = req["collection"].as_str().unwrap_or("").to_string();
            let part = req["partition"].as_str().unwrap_or("").to_string();
            let id   = req["id"].as_str().unwrap_or("").to_string();
            let ctx  = caller.data().ctx.clone();
            ctx.get(&col, &part, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("record '{}' not found", id))
        });
        respond(&mut caller, result)
    })?;

    // qv_insert {"collection","partition","id","fields":{..}}
    linker.func_wrap("qv", "qv_insert", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col    = req["collection"].as_str().unwrap_or("").to_string();
            let part   = req["partition"].as_str().unwrap_or("").to_string();
            let id     = req["id"].as_str().unwrap_or("").to_string();
            let fields = parse_fields(&req["fields"])?;
            let ctx    = caller.data().ctx.clone();
            ctx.insert(&col, &part, &id, fields).map_err(|e| e.to_string())
        });
        respond(&mut caller, result)
    })?;

    // qv_update {"collection","partition","id","fields":{..}}
    linker.func_wrap("qv", "qv_update", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col    = req["collection"].as_str().unwrap_or("").to_string();
            let part   = req["partition"].as_str().unwrap_or("").to_string();
            let id     = req["id"].as_str().unwrap_or("").to_string();
            let fields = parse_fields(&req["fields"])?;
            let ctx    = caller.data().ctx.clone();
            ctx.update(&col, &part, &id, fields).map_err(|e| e.to_string())
        });
        respond(&mut caller, result)
    })?;

    // qv_patch {"collection","partition","id","fields":{..}}
    linker.func_wrap("qv", "qv_patch", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col    = req["collection"].as_str().unwrap_or("").to_string();
            let part   = req["partition"].as_str().unwrap_or("").to_string();
            let id     = req["id"].as_str().unwrap_or("").to_string();
            let fields = parse_fields(&req["fields"])?;
            let ctx    = caller.data().ctx.clone();
            ctx.patch(&col, &part, &id, fields).map_err(|e| e.to_string())
        });
        respond(&mut caller, result)
    })?;

    // qv_delete {"collection","partition","id","reason"?}
    linker.func_wrap("qv", "qv_delete", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col    = req["collection"].as_str().unwrap_or("").to_string();
            let part   = req["partition"].as_str().unwrap_or("").to_string();
            let id     = req["id"].as_str().unwrap_or("").to_string();
            let reason = req["reason"].as_str().map(|s| s.to_string());
            let ctx    = caller.data().ctx.clone();
            ctx.delete(&col, &part, &id, reason).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"deleted": true, "id": id}))
        });
        respond(&mut caller, result)
    })?;

    // qv_restore {"collection","partition","id"}
    linker.func_wrap("qv", "qv_restore", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col  = req["collection"].as_str().unwrap_or("").to_string();
            let part = req["partition"].as_str().unwrap_or("").to_string();
            let id   = req["id"].as_str().unwrap_or("").to_string();
            let ctx  = caller.data().ctx.clone();
            ctx.restore(&col, &part, &id).map_err(|e| e.to_string())
        });
        respond(&mut caller, result)
    })?;

    // qv_upsert {"collection","partition","id","fields":{..}}
    linker.func_wrap("qv", "qv_upsert", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col    = req["collection"].as_str().unwrap_or("").to_string();
            let part   = req["partition"].as_str().unwrap_or("").to_string();
            let id     = req["id"].as_str().unwrap_or("").to_string();
            let fields = parse_fields(&req["fields"])?;
            let ctx    = caller.data().ctx.clone();
            let (rec, action) = ctx.upsert(&col, &part, &id, fields).map_err(|e| e.to_string())?;
            let action_str = format!("{:?}", action).to_lowercase();
            Ok(serde_json::json!({"record": rec, "action": action_str}))
        });
        respond(&mut caller, result)
    })?;

    // qv_query {"collection","partition"?,"filter"?,"sort"?,"limit"?,"page_token"?}
    linker.func_wrap("qv", "qv_query", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col  = req["collection"].as_str().unwrap_or("").to_string();
            let part = req["partition"].as_str().map(|s| s.to_string());

            let filter: Option<Filter> = if req["filter"].is_null() || req["filter"].is_object()
                && req["filter"].as_object().map(|o| o.is_empty()).unwrap_or(true) {
                None
            } else {
                Some(serde_json::from_value(req["filter"].clone()).map_err(|e| e.to_string())?)
            };

            let sort: Option<Vec<SortBy>> = if req["sort"].is_array() {
                Some(serde_json::from_value(req["sort"].clone()).map_err(|e| e.to_string())?)
            } else {
                None
            };

            let limit      = req["limit"].as_u64().unwrap_or(50) as u32;
            let page_token = req["page_token"].as_str().map(|s| s.to_string());
            let pagination = Some(Pagination { limit, page_token });

            let ctx    = caller.data().ctx.clone();
            let result = ctx.query(&col, part.as_deref(), filter, sort, pagination)
                .map_err(|e| e.to_string())?;

            Ok(serde_json::json!({
                "records":    result.records,
                "total":      result.total,
                "page_token": result.page_token,
            }))
        });
        respond(&mut caller, result)
    })?;

    // qv_history {"collection","id"}
    linker.func_wrap("qv", "qv_history", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i64 {
        let req_result = read_str(&mut caller, ptr, len);
        let result = req_result.and_then(|s| {
            let req: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            let col     = req["collection"].as_str().unwrap_or("").to_string();
            let id      = req["id"].as_str().unwrap_or("").to_string();
            let ctx     = caller.data().ctx.clone();
            let entries = ctx.get_history(&col, &id).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"id": id, "history": entries}))
        });
        respond(&mut caller, result)
    })?;

    // qv_emit_event (name_ptr, name_len, payload_ptr, payload_len)
    linker.func_wrap("qv", "qv_emit_event", |mut caller: Caller<'_, WasmState>,
        name_ptr: i32, name_len: i32,
        payload_ptr: i32, payload_len: i32|
    {
        let name    = read_str(&mut caller, name_ptr, name_len).unwrap_or_default();
        let payload = read_bytes(&mut caller, payload_ptr, payload_len).unwrap_or_default();
        caller.data().ctx.emit_event(&name, &payload);
    })?;

    // qv_has_role (role_ptr, role_len) -> i32  (1=true, 0=false)
    linker.func_wrap("qv", "qv_has_role", |mut caller: Caller<'_, WasmState>, ptr: i32, len: i32| -> i32 {
        let role = read_str(&mut caller, ptr, len).unwrap_or_default();
        if caller.data().ctx.has_role(&role) { 1 } else { 0 }
    })?;

    // qv_caller () -> i64  (ptr+len of Identity JSON)
    linker.func_wrap("qv", "qv_caller", |mut caller: Caller<'_, WasmState>| -> i64 {
        let identity = caller.data().ctx.identity();
        let bytes = serde_json::to_vec(&identity).unwrap_or_default();
        write_to_wasm(&mut caller, &bytes)
    })?;

    Ok(())
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn execute_wasm(
    wasm_bytes:    &[u8],
    function_name: &str,
    args:          Value,
    ctx:           Arc<ChainContextImpl>,
) -> Result<ExecResult, String> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes)
        .map_err(|e| format!("Failed to compile WASM: {}", e))?;

    let mut linker: Linker<WasmState> = Linker::new(&engine);
    register(&mut linker).map_err(|e| format!("Failed to register host functions: {}", e))?;

    let mut store = Store::new(&engine, WasmState { ctx });

    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("WASM module must export 'memory'")?;

    let alloc_fn = instance
        .get_typed_func::<i32, i32>(&mut store, "alloc")
        .map_err(|_| "WASM module must export 'alloc(i32) -> i32'")?;

    let dispatch_fn = instance
        .get_typed_func::<(i32, i32, i32, i32), i64>(&mut store, "dispatch")
        .map_err(|_| "WASM module must export 'dispatch(i32,i32,i32,i32) -> i64'")?;

    // Write function_name into WASM memory
    let fn_bytes = function_name.as_bytes();
    let fn_len   = fn_bytes.len() as i32;
    let fn_ptr   = alloc_fn.call(&mut store, fn_len).map_err(|e| e.to_string())?;
    memory.write(&mut store, fn_ptr as usize, fn_bytes).map_err(|e| e.to_string())?;

    // Write args JSON into WASM memory
    let args_bytes = serde_json::to_vec(&args).map_err(|e| e.to_string())?;
    let args_len   = args_bytes.len() as i32;
    let args_ptr   = alloc_fn.call(&mut store, args_len).map_err(|e| e.to_string())?;
    memory.write(&mut store, args_ptr as usize, &args_bytes).map_err(|e| e.to_string())?;

    // Call the contract's dispatch function
    let encoded = dispatch_fn
        .call(&mut store, (fn_ptr, fn_len, args_ptr, args_len))
        .map_err(|e| format!("WASM dispatch error: {}", e))?;

    let result_ptr = (encoded >> 32) as i32;
    let result_len = (encoded & 0xFFFF_FFFF) as i32;

    if result_ptr == 0 || result_len <= 0 {
        return Err("WASM dispatch returned null or empty response".into());
    }

    let result_bytes = {
        let data = memory.data(&store);
        data.get(result_ptr as usize..(result_ptr + result_len) as usize)
            .ok_or("result pointer out of bounds")?
            .to_vec()
    };

    // Unwrap the JSON envelope {"ok":bool,"data":...,"error":"..."}
    let envelope: Value = serde_json::from_slice(&result_bytes)
        .map_err(|e| format!("Failed to parse WASM result envelope: {}", e))?;

    let response = if envelope["ok"].as_bool().unwrap_or(false) {
        envelope["data"].clone()
    } else {
        let msg = envelope["error"].as_str().unwrap_or("unknown contract error");
        return Err(msg.to_string());
    };

    let ctx = &store.data().ctx;
    Ok(ExecResult {
        response,
        events: ctx.sim.drain_events(),
        ops:    ctx.sim.drain_ops(),
        reads:  ctx.sim.drain_reads(),
    })
}
