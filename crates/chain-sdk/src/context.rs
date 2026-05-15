use crate::error::ChainError;
use crate::types::{FieldValue, Filter, HistoryEntry, Identity, UpsertAction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResult {
    pub records:    Vec<serde_json::Value>,
    pub total:      u64,
    pub page_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SortBy {
    pub field:      String,
    pub descending: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub limit:      u32,
    pub page_token: Option<String>,
}

pub trait ChainContext: Send + Sync {
    fn insert(&self, collection: &str, partition: &str, id: &str,
              fields: HashMap<String, FieldValue>) -> Result<serde_json::Value, ChainError>;

    fn get(&self, collection: &str, partition: &str, id: &str)
        -> Result<Option<serde_json::Value>, ChainError>;

    fn update(&self, collection: &str, partition: &str, id: &str,
              fields: HashMap<String, FieldValue>) -> Result<serde_json::Value, ChainError>;

    fn patch(&self, collection: &str, partition: &str, id: &str,
             patches: HashMap<String, FieldValue>) -> Result<serde_json::Value, ChainError>;

    fn delete(&self, collection: &str, partition: &str, id: &str,
              reason: Option<String>) -> Result<(), ChainError>;

    fn restore(&self, collection: &str, partition: &str, id: &str)
        -> Result<serde_json::Value, ChainError>;

    fn query(&self, collection: &str, partition: Option<&str>,
             filter: Option<Filter>, sort: Option<Vec<SortBy>>,
             pagination: Option<Pagination>) -> Result<QueryResult, ChainError>;

    fn get_history(&self, collection: &str, id: &str)
        -> Result<Vec<HistoryEntry>, ChainError>;

    fn batch_insert(&self, collection: &str, partition: &str,
                    records: Vec<(String, HashMap<String, FieldValue>)>)
        -> Result<Vec<serde_json::Value>, ChainError>;

    fn upsert(&self, collection: &str, partition: &str, id: &str,
              fields: HashMap<String, FieldValue>)
        -> Result<(serde_json::Value, UpsertAction), ChainError>;

    fn emit_event(&self, name: &str, payload: &[u8]);
    fn caller_identity(&self) -> &Identity;
    fn has_role(&self, role: &str) -> bool;
}
