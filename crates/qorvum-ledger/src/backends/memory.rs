//! In-memory storage backend — pure Rust, zero compile time.
//! Uses RwLock<HashMap> per column family. For dev and testing ONLY.

use crate::block::Block;
use crate::delta::{delta_prefix, RecordDelta};
use crate::error::LedgerError;
use crate::record::Record;
use crate::store::{LedgerStore, RecordOp};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// Column families as separate BTreeMaps (BTree for ordered prefix scans)
struct Inner {
    world_state:  BTreeMap<String, String>,   // key → JSON(Record)
    secondary_idx:BTreeMap<String, ()>,        // index_key → present
    block_store:  BTreeMap<u64, String>,       // block_num → JSON(Block)
    tx_index:     BTreeMap<[u8; 32], u64>,     // tx_id → block_num
    history_idx:  BTreeMap<String, u64>,       // "col~id~ver" → block_num
    delta_store:  BTreeMap<String, String>,    // "col~id~{ver:016x}" → JSON(RecordDelta)
}

pub struct MemoryStore {
    inner: RwLock<Inner>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Inner {
                world_state:   BTreeMap::new(),
                secondary_idx: BTreeMap::new(),
                block_store:   BTreeMap::new(),
                tx_index:      BTreeMap::new(),
                history_idx:   BTreeMap::new(),
                delta_store:   BTreeMap::new(),
            }),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self { Self::new() }
}

impl LedgerStore for MemoryStore {
    fn get_record(&self, key: &str) -> Result<Option<Record>, LedgerError> {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        match inner.world_state.get(key) {
            None => Ok(None),
            Some(json) => {
                let rec = serde_json::from_str(json)
                    .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                Ok(Some(rec))
            }
        }
    }

    fn put_record(&self, record: &Record) -> Result<(), LedgerError> {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let json = serde_json::to_string(record)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        inner.world_state.insert(record.composite_key(), json);
        Ok(())
    }

    fn delete_record(&self, key: &str) -> Result<(), LedgerError> {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        inner.world_state.remove(key);
        Ok(())
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<Record>, LedgerError> {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let mut results = Vec::new();
        for (k, v) in inner.world_state.range(prefix.to_string()..) {
            if !k.starts_with(prefix) { break; }
            let rec: Record = serde_json::from_str(v)
                .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
            results.push(rec);
        }
        Ok(results)
    }

    fn put_index(&self, index_key: &str) -> Result<(), LedgerError> {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        inner.secondary_idx.insert(index_key.to_string(), ());
        Ok(())
    }

    fn delete_index(&self, index_key: &str) -> Result<(), LedgerError> {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        inner.secondary_idx.remove(index_key);
        Ok(())
    }

    fn scan_index_prefix(&self, prefix: &str) -> Result<Vec<String>, LedgerError> {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let mut results = Vec::new();
        for (k, _) in inner.secondary_idx.range(prefix.to_string()..) {
            if !k.starts_with(prefix) { break; }
            results.push(k.clone());
        }
        Ok(results)
    }

    fn put_block(&self, block: &Block) -> Result<(), LedgerError> {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let json = serde_json::to_string(block)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        inner.block_store.insert(block.header.block_number, json);
        Ok(())
    }

    fn get_block(&self, block_num: u64) -> Result<Option<Block>, LedgerError> {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        match inner.block_store.get(&block_num) {
            None => Ok(None),
            Some(json) => {
                let block = serde_json::from_str(json)
                    .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                Ok(Some(block))
            }
        }
    }

    fn get_latest_block_num(&self) -> Result<Option<u64>, LedgerError> {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        Ok(inner.block_store.keys().last().copied())
    }

    fn put_tx_index(&self, tx_id: &[u8; 32], block_num: u64) -> Result<(), LedgerError> {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        inner.tx_index.insert(*tx_id, block_num);
        Ok(())
    }

    fn get_tx_block(&self, tx_id: &[u8; 32]) -> Result<Option<u64>, LedgerError> {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        Ok(inner.tx_index.get(tx_id).copied())
    }

    fn put_history(&self, collection: &str, id: &str, version: u64, block_num: u64)
        -> Result<(), LedgerError>
    {
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let key = format!("{}~{}~{:020}", collection, id, version);
        inner.history_idx.insert(key, block_num);
        Ok(())
    }

    fn get_history(&self, collection: &str, id: &str)
        -> Result<Vec<(u64, u64)>, LedgerError>
    {
        let inner = self.inner.read().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let prefix = format!("{}~{}~", collection, id);
        let mut results = Vec::new();
        for (k, block_num) in inner.history_idx.range(prefix.clone()..) {
            if !k.starts_with(&prefix) { break; }
            // Parse version from end of key
            let ver_str = k.trim_start_matches(&prefix);
            if let Ok(ver) = ver_str.parse::<u64>() {
                results.push((ver, *block_num));
            }
        }
        Ok(results)
    }

    fn commit_block(&self, block: &Block, record_ops: Vec<RecordOp>)
        -> Result<(), LedgerError>
    {
        // Apply all ops atomically (under write lock for the whole operation)
        let mut inner = self.inner.write().map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;

        // Write block
        let json = serde_json::to_string(block)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        inner.block_store.insert(block.header.block_number, json);

        // Index transactions
        for tx in &block.transactions {
            inner.tx_index.insert(tx.tx_id, block.header.block_number);
        }

        // Apply record operations
        for op in record_ops {
            match op {
                RecordOp::Put(record) => {
                    // Compute delta against previous world state (held under same lock).
                    let prev: Option<Record> = inner.world_state.get(&record.composite_key())
                        .and_then(|j| serde_json::from_str(j).ok());
                    let delta = RecordDelta::compute(prev.as_ref(), &record);
                    let delta_json = serde_json::to_string(&delta)
                        .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                    inner.delta_store.insert(delta.storage_key(), delta_json);

                    // History index
                    let hist_key = format!("{}~{}~{:020}",
                        record.meta.collection, record.meta.id, record.meta.version);
                    inner.history_idx.insert(hist_key, block.header.block_number);

                    // World state (full record for fast current-state reads)
                    let json = serde_json::to_string(&record)
                        .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                    inner.world_state.insert(record.composite_key(), json);
                }
                RecordOp::Delete(key) => {
                    inner.world_state.remove(&key);
                }
            }
        }
        Ok(())
    }

    // ── Delta store ───────────────────────────────────────────────────────────

    fn put_delta(&self, delta: &RecordDelta) -> Result<(), LedgerError> {
        let mut inner = self.inner.write()
            .map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let json = serde_json::to_string(delta)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        inner.delta_store.insert(delta.storage_key(), json);
        Ok(())
    }

    fn get_deltas(&self, collection: &str, id: &str)
        -> Result<Vec<RecordDelta>, LedgerError>
    {
        let inner = self.inner.read()
            .map_err(|_| LedgerError::StorageError("lock poisoned".into()))?;
        let prefix = delta_prefix(collection, id);
        let mut results = Vec::new();
        for (k, v) in inner.delta_store.range(prefix.clone()..) {
            if !k.starts_with(&prefix) { break; }
            let delta: RecordDelta = serde_json::from_str(v)
                .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
            results.push(delta);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{RecordMeta, FieldValue};
    use std::collections::HashMap;

    fn make_record(id: &str, collection: &str, partition: &str) -> Record {
        Record {
            meta: RecordMeta {
                id: id.to_string(),
                collection: collection.to_string(),
                partition: partition.to_string(),
                version: 1,
                created_at: 0,
                updated_at: 0,
                created_by: "test".into(),
                updated_by: "test".into(),
                is_deleted: false,
                delete_reason: None,
                tx_id: [0u8; 32],
                block_num: 1,
            },
            fields: {
                let mut m = HashMap::new();
                m.insert("name".into(), FieldValue::Text("Test User".into()));
                m
            },
        }
    }

    #[test]
    fn test_put_get_record() {
        let store = MemoryStore::new();
        let rec = make_record("EMP001", "employees", "HR");
        store.put_record(&rec).unwrap();
        let got = store.get_record("employees~HR~EMP001").unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().meta.id, "EMP001");
    }

    #[test]
    fn test_scan_prefix() {
        let store = MemoryStore::new();
        store.put_record(&make_record("EMP001", "employees", "HR")).unwrap();
        store.put_record(&make_record("EMP002", "employees", "HR")).unwrap();
        store.put_record(&make_record("EMP100", "employees", "IT")).unwrap();

        let hr_records = store.scan_prefix("employees~HR~").unwrap();
        assert_eq!(hr_records.len(), 2);
        let all_records = store.scan_prefix("employees~").unwrap();
        assert_eq!(all_records.len(), 3);
    }

    #[test]
    fn test_index_ops() {
        let store = MemoryStore::new();
        store.put_index("employees~status~ACTIVE~EMP001").unwrap();
        store.put_index("employees~status~ACTIVE~EMP002").unwrap();
        store.put_index("employees~status~INACTIVE~EMP003").unwrap();

        let active = store.scan_index_prefix("employees~status~ACTIVE~").unwrap();
        assert_eq!(active.len(), 2);
    }
}
