use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum FieldValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(u64),
    Json(serde_json::Value),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Filter {
    Eq(String, FieldValue),
    Neq(String, FieldValue),
    Gt(String, FieldValue),
    Lt(String, FieldValue),
    Gte(String, FieldValue),
    Lte(String, FieldValue),
    Between(String, FieldValue, FieldValue),
    In(String, Vec<FieldValue>),
    IsNull(String),
    IsNotNull(String),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
    IncludeDeleted,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub version:    u64,
    pub tx_id:      [u8; 32],
    pub block_num:  u64,
    pub timestamp:  u64,
    pub updated_by: String,
    pub fields:     HashMap<String, FieldValue>,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Identity {
    pub id: String,
    /// Human-readable org name (e.g. "Org1")
    pub msp_id: String,
    pub roles: Vec<String>,
    /// Caller display name, e.g. "alice"
    pub subject: String,
    /// Organization, e.g. "Org1"
    pub org: String,
    /// "User" | "Node" | "CA"
    pub cert_type: String,
    pub email: Option<String>,
    /// false in dev mode, true when cert was cryptographically verified
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpsertAction { Inserted, Updated }

use crate::error::ChainError;
