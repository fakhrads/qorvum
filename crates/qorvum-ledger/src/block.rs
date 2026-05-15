//! Block and Transaction structures for Qorvum.

use crate::record::FieldValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A read captured during simulation (for MVCC validation at commit)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KVRead {
    pub key:             String,
    pub version_at_read: u64,
}

/// A write captured during simulation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KVWrite {
    pub key:       String,
    pub value:     Option<String>, // JSON-encoded Record, None = delete
    pub is_delete: bool,
}

/// The simulation result: what was read and what will be written
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReadWriteSet {
    pub reads:  Vec<KVRead>,
    pub writes: Vec<KVWrite>,
}

/// Signature structure (algorithm-tagged)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EndorsementSig {
    pub algorithm: String,   // "dilithium3" | "falcon512"
    pub bytes:     Vec<u8>,
}

/// An endorsement = peer's signed approval of a simulation result
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Endorsement {
    pub endorser_msp_id:  String,
    pub endorser_pub_key: Vec<u8>,
    pub signature:        EndorsementSig,
    pub read_write_set:   ReadWriteSet,
    pub response_payload: Vec<u8>,   // contract return value
}

/// A single transaction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub tx_id:         [u8; 32],         // BLAKE3(canonical fields)
    pub channel_id:    String,
    pub contract_id:   String,
    pub function_name: String,
    pub args:          serde_json::Value, // JSON-encoded args
    pub creator_pub_key: Vec<u8>,
    pub creator_sig:   EndorsementSig,
    pub endorsements:  Vec<Endorsement>,
    pub nonce:         [u8; 32],
    pub timestamp:     u64,              // Unix nanos
}

/// Block header (this is what gets hashed for chain linking)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub version:           u32,
    pub block_number:      u64,
    pub previous_hash:     [u8; 32],
    pub transactions_root: [u8; 32],
    pub state_root:        [u8; 32],
    pub timestamp:         u64,
    pub channel_id:        String,
    pub creator_msp_id:    String,
    pub creator_pub_key:   Vec<u8>,
    pub creator_sig:       EndorsementSig,
}

/// Metadata appended by committer after the block is written
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockMetadata {
    pub block_hash: [u8; 32],
    pub tx_count:   u32,
}

/// A full, committed block
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header:       BlockHeader,
    pub transactions: Vec<Transaction>,
    pub metadata:     BlockMetadata,
}

impl Block {
    pub fn compute_hash(&self) -> [u8; 32] {
        let bytes = serde_json::to_vec(&self.header).unwrap_or_default();
        qorvum_crypto::hash(&bytes)
    }
}

/// Builder for constructing a block from collected transactions
pub struct BlockBuilder {
    pub channel_id:     String,
    pub block_number:   u64,
    pub previous_hash:  [u8; 32],
    pub transactions:   Vec<Transaction>,
    pub creator_msp_id: String,
    pub creator_pub_key:Vec<u8>,
    pub creator_sig:    EndorsementSig,
}

impl BlockBuilder {
    pub fn new(
        channel_id:     String,
        block_number:   u64,
        previous_hash:  [u8; 32],
        creator_msp_id: String,
        creator_pub_key:Vec<u8>,
        creator_sig:    EndorsementSig,
    ) -> Self {
        Self {
            channel_id,
            block_number,
            previous_hash,
            transactions: Vec::new(),
            creator_msp_id,
            creator_pub_key,
            creator_sig,
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
    }

    pub fn build(self) -> Block {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Compute Merkle root of transaction IDs
        let tx_root = Self::merkle_root(
            &self.transactions.iter().map(|t| t.tx_id).collect::<Vec<_>>()
        );

        let header = BlockHeader {
            version:           1,
            block_number:      self.block_number,
            previous_hash:     self.previous_hash,
            transactions_root: tx_root,
            state_root:        [0u8; 32], // updated after world state commit
            timestamp:         now,
            channel_id:        self.channel_id,
            creator_msp_id:    self.creator_msp_id,
            creator_pub_key:   self.creator_pub_key,
            creator_sig:       self.creator_sig,
        };

        let tx_count = self.transactions.len() as u32;
        let block = Block {
            header,
            transactions: self.transactions,
            metadata: BlockMetadata { block_hash: [0u8; 32], tx_count },
        };

        let hash = block.compute_hash();
        Block {
            metadata: BlockMetadata { block_hash: hash, tx_count },
            ..block
        }
    }

    fn merkle_root(ids: &[[u8; 32]]) -> [u8; 32] {
        if ids.is_empty() { return [0u8; 32]; }
        if ids.len() == 1 { return ids[0]; }
        let mut layer: Vec<[u8; 32]> = ids.to_vec();
        while layer.len() > 1 {
            if layer.len() % 2 == 1 { layer.push(*layer.last().unwrap()); }
            layer = layer.chunks(2)
                .map(|pair| qorvum_crypto::hash_many(&[&pair[0], &pair[1]]))
                .collect();
        }
        layer[0]
    }
}
