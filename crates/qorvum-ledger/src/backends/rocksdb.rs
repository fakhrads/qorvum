//! RocksDB persistent storage backend.
//! Uses 5 column families so each namespace is isolated and prefix-scannable.
//!
//! Column Families:
//!   CF_WORLD    — "world_state"   : key(collection~partition~id) → JSON(Record)
//!   CF_IDX      — "secondary_idx" : index_key → "" (presence only)
//!   CF_BLOCKS   — "block_store"   : big-endian u64(block_num) → JSON(Block)
//!   CF_TX       — "tx_index"      : hex(tx_id[32]) → big-endian u64(block_num)
//!   CF_HISTORY  — "history_idx"   : "col~id~{:020}" → big-endian u64(block_num)

use crate::block::Block;
use crate::error::LedgerError;
use crate::record::Record;
use crate::store::{LedgerStore, RecordOp};
use rocksdb::{ColumnFamilyDescriptor, DB, Options, WriteBatch};
use std::path::Path;

// ── Column Family Names ───────────────────────────────────────────────────────
const CF_WORLD:   &str = "world_state";
const CF_IDX:     &str = "secondary_idx";
const CF_BLOCKS:  &str = "block_store";
const CF_TX:      &str = "tx_index";
const CF_HISTORY: &str = "history_idx";

pub struct RocksDbStore {
    db: DB,
}

impl RocksDbStore {
    /// Open (or create) the RocksDB database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, LedgerError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_names = [CF_WORLD, CF_IDX, CF_BLOCKS, CF_TX, CF_HISTORY];
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|&name| ColumnFamilyDescriptor::new(name, Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| LedgerError::StorageError(e.to_string()))?;

        Ok(Self { db })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn cf_world(&self) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(CF_WORLD).expect("CF world_state missing")
    }

    fn cf_idx(&self) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(CF_IDX).expect("CF secondary_idx missing")
    }

    fn cf_blocks(&self) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(CF_BLOCKS).expect("CF block_store missing")
    }

    fn cf_tx(&self) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(CF_TX).expect("CF tx_index missing")
    }

    fn cf_history(&self) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(CF_HISTORY).expect("CF history_idx missing")
    }

    fn encode_block_num(n: u64) -> [u8; 8] {
        n.to_be_bytes()
    }

    fn decode_block_num(bytes: &[u8]) -> Result<u64, LedgerError> {
        bytes.try_into()
            .map(u64::from_be_bytes)
            .map_err(|_| LedgerError::StorageError("corrupt block_num encoding".into()))
    }

    fn history_key(collection: &str, id: &str, version: u64) -> String {
        format!("{}~{}~{:020}", collection, id, version)
    }

    fn history_prefix(collection: &str, id: &str) -> String {
        format!("{}~{}~", collection, id)
    }
}

// ── LedgerStore impl ──────────────────────────────────────────────────────────

impl LedgerStore for RocksDbStore {
    // ── World State ───────────────────────────────────────────────────────────

    fn get_record(&self, key: &str) -> Result<Option<Record>, LedgerError> {
        let cf = self.cf_world();
        match self.db.get_cf(cf, key.as_bytes())
            .map_err(|e| LedgerError::StorageError(e.to_string()))?
        {
            None => Ok(None),
            Some(bytes) => {
                let rec: Record = serde_json::from_slice(&bytes)
                    .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                Ok(Some(rec))
            }
        }
    }

    fn put_record(&self, record: &Record) -> Result<(), LedgerError> {
        let cf = self.cf_world();
        let json = serde_json::to_vec(record)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, record.composite_key().as_bytes(), &json)
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn delete_record(&self, key: &str) -> Result<(), LedgerError> {
        let cf = self.cf_world();
        self.db.delete_cf(cf, key.as_bytes())
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<Record>, LedgerError> {
        let cf = self.cf_world();
        let iter = self.db.prefix_iterator_cf(cf, prefix.as_bytes());
        let mut results = Vec::new();
        for item in iter {
            let (k, v) = item.map_err(|e| LedgerError::StorageError(e.to_string()))?;
            let key_str = std::str::from_utf8(&k)
                .map_err(|_| LedgerError::StorageError("invalid UTF-8 key".into()))?;
            if !key_str.starts_with(prefix) {
                break;
            }
            let rec: Record = serde_json::from_slice(&v)
                .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
            results.push(rec);
        }
        Ok(results)
    }

    // ── Secondary Index ───────────────────────────────────────────────────────

    fn put_index(&self, index_key: &str) -> Result<(), LedgerError> {
        let cf = self.cf_idx();
        self.db.put_cf(cf, index_key.as_bytes(), b"")
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn delete_index(&self, index_key: &str) -> Result<(), LedgerError> {
        let cf = self.cf_idx();
        self.db.delete_cf(cf, index_key.as_bytes())
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn scan_index_prefix(&self, prefix: &str) -> Result<Vec<String>, LedgerError> {
        let cf = self.cf_idx();
        let iter = self.db.prefix_iterator_cf(cf, prefix.as_bytes());
        let mut results = Vec::new();
        for item in iter {
            let (k, _) = item.map_err(|e| LedgerError::StorageError(e.to_string()))?;
            let key_str = std::str::from_utf8(&k)
                .map_err(|_| LedgerError::StorageError("invalid UTF-8 key".into()))?;
            if !key_str.starts_with(prefix) {
                break;
            }
            results.push(key_str.to_string());
        }
        Ok(results)
    }

    // ── Block Store ───────────────────────────────────────────────────────────

    fn put_block(&self, block: &Block) -> Result<(), LedgerError> {
        let cf = self.cf_blocks();
        let key = Self::encode_block_num(block.header.block_number);
        let json = serde_json::to_vec(block)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        self.db.put_cf(cf, &key, &json)
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn get_block(&self, block_num: u64) -> Result<Option<Block>, LedgerError> {
        let cf = self.cf_blocks();
        let key = Self::encode_block_num(block_num);
        match self.db.get_cf(cf, &key)
            .map_err(|e| LedgerError::StorageError(e.to_string()))?
        {
            None => Ok(None),
            Some(bytes) => {
                let block: Block = serde_json::from_slice(&bytes)
                    .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                Ok(Some(block))
            }
        }
    }

    fn get_latest_block_num(&self) -> Result<Option<u64>, LedgerError> {
        let cf = self.cf_blocks();
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek_to_last();
        if iter.valid() {
            let key = iter.key().ok_or_else(|| LedgerError::StorageError("invalid iter key".into()))?;
            let num = Self::decode_block_num(key)?;
            Ok(Some(num))
        } else {
            Ok(None)
        }
    }

    // ── TX Index ──────────────────────────────────────────────────────────────

    fn put_tx_index(&self, tx_id: &[u8; 32], block_num: u64) -> Result<(), LedgerError> {
        let cf = self.cf_tx();
        let key = hex::encode(tx_id);
        let val = Self::encode_block_num(block_num);
        self.db.put_cf(cf, key.as_bytes(), &val)
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn get_tx_block(&self, tx_id: &[u8; 32]) -> Result<Option<u64>, LedgerError> {
        let cf = self.cf_tx();
        let key = hex::encode(tx_id);
        match self.db.get_cf(cf, key.as_bytes())
            .map_err(|e| LedgerError::StorageError(e.to_string()))?
        {
            None => Ok(None),
            Some(bytes) => Ok(Some(Self::decode_block_num(&bytes)?)),
        }
    }

    // ── History Index ─────────────────────────────────────────────────────────

    fn put_history(&self, collection: &str, id: &str, version: u64, block_num: u64)
        -> Result<(), LedgerError>
    {
        let cf = self.cf_history();
        let key = Self::history_key(collection, id, version);
        let val = Self::encode_block_num(block_num);
        self.db.put_cf(cf, key.as_bytes(), &val)
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn get_history(&self, collection: &str, id: &str)
        -> Result<Vec<(u64, u64)>, LedgerError>
    {
        let cf = self.cf_history();
        let prefix = Self::history_prefix(collection, id);
        let iter = self.db.prefix_iterator_cf(cf, prefix.as_bytes());
        let mut results = Vec::new();
        for item in iter {
            let (k, v) = item.map_err(|e| LedgerError::StorageError(e.to_string()))?;
            let key_str = std::str::from_utf8(&k)
                .map_err(|_| LedgerError::StorageError("invalid UTF-8 key".into()))?;
            if !key_str.starts_with(&prefix) {
                break;
            }
            let ver_str = key_str.trim_start_matches(&prefix);
            if let Ok(ver) = ver_str.parse::<u64>() {
                let block_num = Self::decode_block_num(&v)?;
                results.push((ver, block_num));
            }
        }
        Ok(results)
    }

    // ── Atomic Commit ─────────────────────────────────────────────────────────

    fn commit_block(&self, block: &Block, record_ops: Vec<RecordOp>)
        -> Result<(), LedgerError>
    {
        let mut batch = WriteBatch::default();

        // Write block to CF_BLOCKS
        let block_key = Self::encode_block_num(block.header.block_number);
        let block_json = serde_json::to_vec(block)
            .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
        batch.put_cf(self.cf_blocks(), &block_key, &block_json);

        // Index all transactions into CF_TX
        for tx in &block.transactions {
            let tx_key = hex::encode(&tx.tx_id);
            let block_num_bytes = Self::encode_block_num(block.header.block_number);
            batch.put_cf(self.cf_tx(), tx_key.as_bytes(), &block_num_bytes);
        }

        // Apply world state + history changes
        for op in record_ops {
            match op {
                RecordOp::Put(record) => {
                    // History entry
                    let hist_key = Self::history_key(
                        &record.meta.collection, &record.meta.id, record.meta.version,
                    );
                    let block_num_bytes = Self::encode_block_num(block.header.block_number);
                    batch.put_cf(self.cf_history(), hist_key.as_bytes(), &block_num_bytes);

                    // World state entry
                    let record_json = serde_json::to_vec(&record)
                        .map_err(|e| LedgerError::SerializationError(e.to_string()))?;
                    batch.put_cf(self.cf_world(), record.composite_key().as_bytes(), &record_json);
                }
                RecordOp::Delete(key) => {
                    batch.delete_cf(self.cf_world(), key.as_bytes());
                }
            }
        }

        // Flush the entire batch atomically
        self.db.write(batch)
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }
}
