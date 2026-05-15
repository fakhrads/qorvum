//! LedgerStore trait + RecordOp enum
//! Two backends:
//!   - MemoryStore  (default, zero compile, dev/test)
//!   - RocksDbStore (feature = "rocksdb-store", production)

use crate::block::Block;
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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RecordOp {
    Put(Record),
    Delete(String),
}
