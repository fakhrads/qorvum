//! WorldState — high-level CRUD engine that sits on top of LedgerStore.
//! This is what the contract executor calls. It collects a ReadWriteSet
//! during simulation, then applies it atomically at commit time.

use crate::error::LedgerError;
use crate::record::{FieldValue, Record, RecordMeta};
use crate::store::{LedgerStore, RecordOp};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// A pending write collected during tx simulation
#[derive(Debug, Clone)]
pub enum PendingWrite {
    Put(Record),
    Delete(String), // composite key
}

/// Simulation context: reads from store, buffers writes
/// All writes only become visible after `flush_to_block()`
pub struct SimulationContext {
    store:         Arc<dyn LedgerStore>,
    reads:         Mutex<Vec<(String, u64)>>,    // (key, version_read)
    writes:        Mutex<Vec<PendingWrite>>,
    events:        Mutex<Vec<(String, Vec<u8>)>>,
    caller_id:     String,
    caller_msp:    String,
    caller_roles:  Vec<String>,
    tx_id:         [u8; 32],
    timestamp:     u64,
}

impl SimulationContext {
    pub fn new(
        store:        Arc<dyn LedgerStore>,
        caller_id:    String,
        caller_msp:   String,
        caller_roles: Vec<String>,
        tx_id:        [u8; 32],
        timestamp:    u64,
    ) -> Self {
        Self {
            store,
            reads:        Mutex::new(Vec::new()),
            writes:       Mutex::new(Vec::new()),
            events:       Mutex::new(Vec::new()),
            caller_id,
            caller_msp,
            caller_roles,
            tx_id,
            timestamp,
        }
    }

    pub fn caller_id(&self)   -> &str { &self.caller_id }
    pub fn caller_msp(&self)  -> &str { &self.caller_msp }
    pub fn has_role(&self, role: &str) -> bool {
        self.caller_roles.iter().any(|r| r == role)
    }

    // ── READ ─────────────────────────────────────────────────────────────────

    pub fn get(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
    ) -> Result<Option<Record>, LedgerError> {
        let key = format!("{}~{}~{}", collection, partition, id);

        // Check local write buffer first (read-your-own-writes)
        {
            let writes = self.writes.lock().unwrap();
            for w in writes.iter().rev() {
                match w {
                    PendingWrite::Put(r) if r.composite_key() == key => {
                        // Track read version from pending write
                        let mut reads = self.reads.lock().unwrap();
                        reads.push((key.clone(), r.meta.version));
                        return Ok(Some(r.clone()));
                    }
                    PendingWrite::Delete(k) if *k == key => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }

        // Read from store
        let result = self.store.get_record(&key)?;
        let version = result.as_ref().map(|r| r.meta.version).unwrap_or(0);
        self.reads.lock().unwrap().push((key, version));
        Ok(result)
    }

    pub fn scan(
        &self,
        collection: &str,
        partition:  Option<&str>,
    ) -> Result<Vec<Record>, LedgerError> {
        let prefix = match partition {
            Some(p) => format!("{}~{}~", collection, p),
            None    => format!("{}~", collection),
        };
        self.store.scan_prefix(&prefix)
    }

    pub fn get_history(
        &self,
        collection: &str,
        id:         &str,
    ) -> Result<Vec<(u64, u64)>, LedgerError> {
        self.store.get_history(collection, id)
    }

    // ── WRITE ────────────────────────────────────────────────────────────────

    pub fn insert(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        fields:     HashMap<String, FieldValue>,
    ) -> Result<Record, LedgerError> {
        let key = format!("{}~{}~{}", collection, partition, id);

        // Guard: must not already exist
        if self.store.get_record(&key)?.is_some() {
            return Err(LedgerError::AlreadyExists(key));
        }
        // Also check write buffer
        {
            let writes = self.writes.lock().unwrap();
            for w in writes.iter() {
                if let PendingWrite::Put(r) = w {
                    if r.composite_key() == key {
                        return Err(LedgerError::AlreadyExists(key));
                    }
                }
            }
        }

        let record = Record {
            meta: RecordMeta {
                id:            id.to_string(),
                collection:    collection.to_string(),
                partition:     partition.to_string(),
                version:       1,
                created_at:    self.timestamp,
                updated_at:    self.timestamp,
                created_by:    self.caller_id.clone(),
                updated_by:    self.caller_id.clone(),
                is_deleted:    false,
                delete_reason: None,
                tx_id:         self.tx_id,
                block_num:     0, // filled at commit
            },
            fields,
        };

        debug!("insert: {}", key);
        self.writes.lock().unwrap().push(PendingWrite::Put(record.clone()));
        self._index_write(&record);
        Ok(record)
    }

    pub fn update(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        fields:     HashMap<String, FieldValue>,
    ) -> Result<Record, LedgerError> {
        let existing = self.get(collection, partition, id)?
            .ok_or_else(|| LedgerError::NotFound(format!("{}~{}~{}", collection, partition, id)))?;

        self._index_remove(&existing);
        let record = Record {
            meta: RecordMeta {
                version:    existing.meta.version + 1,
                updated_at: self.timestamp,
                updated_by: self.caller_id.clone(),
                tx_id:      self.tx_id,
                ..existing.meta
            },
            fields,
        };

        self.writes.lock().unwrap().push(PendingWrite::Put(record.clone()));
        self._index_write(&record);
        Ok(record)
    }

    pub fn patch(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        patches:    HashMap<String, FieldValue>,
    ) -> Result<Record, LedgerError> {
        let mut existing = self.get(collection, partition, id)?
            .ok_or_else(|| LedgerError::NotFound(format!("{}~{}~{}", collection, partition, id)))?;

        self._index_remove(&existing);
        for (k, v) in patches {
            existing.fields.insert(k, v);
        }
        existing.meta.version    += 1;
        existing.meta.updated_at  = self.timestamp;
        existing.meta.updated_by  = self.caller_id.clone();
        existing.meta.tx_id       = self.tx_id;

        self.writes.lock().unwrap().push(PendingWrite::Put(existing.clone()));
        self._index_write(&existing);
        Ok(existing)
    }

    pub fn soft_delete(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
        reason:     Option<String>,
    ) -> Result<(), LedgerError> {
        let mut existing = self.get(collection, partition, id)?
            .ok_or_else(|| LedgerError::NotFound(format!("{}~{}~{}", collection, partition, id)))?;

        self._index_remove(&existing);
        existing.meta.is_deleted    = true;
        existing.meta.delete_reason = reason;
        existing.meta.version      += 1;
        existing.meta.updated_at    = self.timestamp;
        existing.meta.updated_by    = self.caller_id.clone();
        existing.meta.tx_id         = self.tx_id;

        self.writes.lock().unwrap().push(PendingWrite::Put(existing));
        Ok(())
    }

    pub fn restore(
        &self,
        collection: &str,
        partition:  &str,
        id:         &str,
    ) -> Result<Record, LedgerError> {
        let mut existing = self.get(collection, partition, id)?
            .ok_or_else(|| LedgerError::NotFound(format!("{}~{}~{}", collection, partition, id)))?;

        existing.meta.is_deleted    = false;
        existing.meta.delete_reason = None;
        existing.meta.version      += 1;
        existing.meta.updated_at    = self.timestamp;
        existing.meta.updated_by    = self.caller_id.clone();
        existing.meta.tx_id         = self.tx_id;

        self.writes.lock().unwrap().push(PendingWrite::Put(existing.clone()));
        self._index_write(&existing);
        Ok(existing)
    }

    pub fn emit_event(&self, name: &str, payload: Vec<u8>) {
        self.events.lock().unwrap().push((name.to_string(), payload));
    }

    // ── DRAIN (called by executor after simulation) ───────────────────────────

    pub fn drain_ops(&self) -> Vec<RecordOp> {
        self.writes.lock().unwrap().drain(..)
            .map(|w| match w {
                PendingWrite::Put(r)    => RecordOp::Put(r),
                PendingWrite::Delete(k) => RecordOp::Delete(k),
            })
            .collect()
    }

    pub fn drain_events(&self) -> Vec<(String, Vec<u8>)> {
        self.events.lock().unwrap().drain(..).collect()
    }

    pub fn drain_reads(&self) -> Vec<(String, u64)> {
        self.reads.lock().unwrap().drain(..).collect()
    }

    pub fn get_pending_writes(&self) -> Vec<PendingWrite> {
        self.writes.lock().unwrap().clone()
    }

    // ── PRIVATE ──────────────────────────────────────────────────────────────

    fn _index_write(&self, record: &Record) {
        // Build secondary index entries for all indexed fields
        // Skipped during simulation — indexes applied at commit via store
        // (in full impl: collect index ops alongside record ops)
        let _ = record; // suppress unused warning for now
    }

    fn _index_remove(&self, record: &Record) {
        let _ = record;
    }
}
