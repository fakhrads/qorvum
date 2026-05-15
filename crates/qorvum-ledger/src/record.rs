//! Core data model — Record, FieldValue, Schema.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All supported field types in a Qorvum record.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum FieldValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(u64),              // Unix nanoseconds
    Json(serde_json::Value),     // Nested / unstructured
}

impl std::fmt::Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Null         => write!(f, "null"),
            FieldValue::Bool(b)      => write!(f, "{}", b),
            FieldValue::Int(n)       => write!(f, "{}", n),
            FieldValue::Float(n)     => write!(f, "{}", n),
            FieldValue::Text(s)      => write!(f, "{}", s),
            FieldValue::Bytes(b)     => write!(f, "0x{}", hex::encode(b)),
            FieldValue::Timestamp(t) => write!(f, "{}", t),
            FieldValue::Json(v)      => write!(f, "{}", v),
        }
    }
}

/// System metadata present on every record — managed by Qorvum, not by contracts.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordMeta {
    pub id:            String,
    pub collection:    String,
    pub partition:     String,    // physical key namespace (e.g. dept, region)
    pub version:       u64,       // increments on every write
    pub created_at:    u64,       // Unix nanos
    pub updated_at:    u64,
    pub created_by:    String,    // MSP identity (e.g. "user:alice@org1")
    pub updated_by:    String,
    pub is_deleted:    bool,      // soft delete flag
    pub delete_reason: Option<String>,
    pub tx_id:         [u8; 32],  // last tx that touched this record
    pub block_num:     u64,       // block that committed this version
}

/// A full record = system metadata + contract-defined business fields.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    pub meta:   RecordMeta,
    pub fields: HashMap<String, FieldValue>,
}

impl Record {
    /// RocksDB / BTreeMap key: `{collection}~{partition}~{id}`
    pub fn composite_key(&self) -> String {
        format!("{}~{}~{}", self.meta.collection, self.meta.partition, self.meta.id)
    }

    /// Secondary index key for a given field value.
    /// Zero-padded ints allow range scans: `employees~salary~00025000000~EMP001`
    pub fn index_key(collection: &str, field: &str, value: &FieldValue, id: &str) -> String {
        let val_str = match value {
            FieldValue::Text(s)        => s.clone(),
            FieldValue::Int(n)         => format!("{:020}", n),
            FieldValue::Bool(b)        => b.to_string(),
            FieldValue::Timestamp(t)   => format!("{:020}", t),
            FieldValue::Float(f)       => format!("{:020.6}", f),
            _                          => "~".to_string(),
        };
        format!("{}~{}~{}~{}", collection, field, val_str, id)
    }
}

// ── Schema ───────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FieldType { Bool, Int, Float, Text, Bytes, Timestamp, Json }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldSchema {
    pub name:       String,
    pub field_type: FieldType,
    pub required:   bool,
    pub indexed:    bool,
    pub unique:     bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CollectionSchema {
    pub name:         String,
    pub contract_id: String,
    pub fields:       Vec<FieldSchema>,
    pub version:      u32,
}

impl CollectionSchema {
    pub fn validate(&self, fields: &HashMap<String, FieldValue>) -> Result<(), String> {
        for schema_field in &self.fields {
            if schema_field.required {
                match fields.get(&schema_field.name) {
                    None | Some(FieldValue::Null) => {
                        return Err(format!("Required field '{}' is missing", schema_field.name));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
