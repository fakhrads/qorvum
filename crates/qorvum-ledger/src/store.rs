//! LedgerStore trait + RecordOp enum
//! Two backends:
//!   - MemoryStore  (default, zero compile, dev/test)
//!   - RocksDbStore (feature = "rocksdb-store", production)

use crate::block::Block;
use crate::delta::RecordDelta;
use crate::error::LedgerError;
use crate::record::Record;

pub trait LedgerStore: Send + Sync {
    fn get_record(&self, key: &str) -> Result<Option<Record>, LedgerError>;
    fn put_record(&self, record: &Record) -> Result<(), LedgerError>;
    fn delete_record(&self, key: &str) -> Result<(), LedgerError>;
    fn scan_prefix(&self, prefix: &str) -> Result<Vec<Record>, LedgerError>;

    fn put_index(&self, index_key: &str) -> Result<(), LedgerError>;
    fn delete_index(&self, index_key: &str) -> Result<(), LedgerError>;
    fn scan_index_prefix(&self, prefix: &str) -> Result<Vec<String>, LedgerError>;

    fn put_block(&self, block: &Block) -> Result<(), LedgerError>;
    fn get_block(&self, block_num: u64) -> Result<Option<Block>, LedgerError>;
    fn get_latest_block_num(&self) -> Result<Option<u64>, LedgerError>;

    fn put_tx_index(&self, tx_id: &[u8; 32], block_num: u64) -> Result<(), LedgerError>;
    fn get_tx_block(&self, tx_id: &[u8; 32]) -> Result<Option<u64>, LedgerError>;

    fn put_history(&self, collection: &str, id: &str, version: u64, block_num: u64)
        -> Result<(), LedgerError>;
    fn get_history(&self, collection: &str, id: &str)
        -> Result<Vec<(u64, u64)>, LedgerError>;

    fn commit_block(&self, block: &Block, record_ops: Vec<RecordOp>)
        -> Result<(), LedgerError>;

    // ── Delta store ───────────────────────────────────────────────────────────

    /// Persist a single delta to `delta_store`.
    /// Called automatically by `commit_block` for every `RecordOp::Put`.
    fn put_delta(&self, delta: &RecordDelta) -> Result<(), LedgerError>;

    /// Return all deltas for a record, sorted oldest → newest by `to_version`.
    fn get_deltas(&self, collection: &str, id: &str)
        -> Result<Vec<RecordDelta>, LedgerError>;

    /// Return all deltas as a structured history (alias of `get_deltas`).
    fn get_record_history_with_delta(&self, collection: &str, id: &str)
        -> Result<Vec<RecordDelta>, LedgerError>
    {
        self.get_deltas(collection, id)
    }

    /// Reconstruct the record state at `target_version` by replaying the delta chain.
    /// Returns `None` if no deltas exist for the given record.
    fn reconstruct_at_version(
        &self,
        collection: &str,
        _partition: &str,
        id: &str,
        target_version: u64,
    ) -> Result<Option<Record>, LedgerError> {
        let deltas = self.get_deltas(collection, id)?;
        if deltas.is_empty() {
            return Ok(None);
        }
        let mut record: Option<Record> = None;
        for delta in deltas.iter().filter(|d| d.to_version <= target_version) {
            record = Some(RecordDelta::apply(record, delta)?);
        }
        Ok(record)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RecordOp {
    Put(Record),
    Delete(String),
}
