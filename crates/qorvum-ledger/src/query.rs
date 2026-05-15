//! Query engine — filter, sort, paginate over world state records.

use crate::error::LedgerError;
use crate::record::{FieldValue, Record};
use crate::store::LedgerStore;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Filter {
    Eq(String, FieldValue),
    Neq(String, FieldValue),
    Gt(String, FieldValue),
    Lt(String, FieldValue),
    Gte(String, FieldValue),
    Lte(String, FieldValue),
    In(String, Vec<FieldValue>),
    IsNull(String),
    IsNotNull(String),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
    IncludeDeleted,
}

#[derive(Debug, Clone)]
pub struct SortBy {
    pub field:      String,
    pub descending: bool,
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit:  usize,
    pub offset: usize,
}

impl Default for Pagination {
    fn default() -> Self { Self { limit: 20, offset: 0 } }
}

#[derive(Debug)]
pub struct QueryResult {
    pub records: Vec<Record>,
    pub total:   usize,     // total matching (before pagination)
    pub offset:  usize,
    pub limit:   usize,
}

pub struct QueryEngine {
    store: Arc<dyn LedgerStore>,
}

impl QueryEngine {
    pub fn new(store: Arc<dyn LedgerStore>) -> Self { Self { store } }

    pub fn query(
        &self,
        collection: &str,
        partition:  Option<&str>,
        filter:     Option<&Filter>,
        sort:       Option<&[SortBy]>,
        pagination: Option<&Pagination>,
    ) -> Result<QueryResult, LedgerError> {
        // 1. Fetch all candidates via prefix scan
        let prefix = match partition {
            Some(p) => format!("{}~{}~", collection, p),
            None    => format!("{}~", collection),
        };
        let mut records = self.store.scan_prefix(&prefix)?;

        // 2. Default: exclude soft-deleted unless IncludeDeleted in filter
        let include_deleted = filter.as_ref()
            .map(|f| has_include_deleted(f))
            .unwrap_or(false);
        if !include_deleted {
            records.retain(|r| !r.meta.is_deleted);
        }

        // 3. Apply filter
        if let Some(f) = filter {
            records.retain(|r| apply_filter(r, f));
        }

        let total = records.len();

        // 4. Sort
        if let Some(sorts) = sort {
            records.sort_by(|a, b| {
                for s in sorts {
                    let ord = compare_field(a, b, &s.field);
                    let ord = if s.descending { ord.reverse() } else { ord };
                    if ord != std::cmp::Ordering::Equal { return ord; }
                }
                std::cmp::Ordering::Equal
            });
        }

        // 5. Paginate
        let pag = pagination.cloned().unwrap_or_default();
        let records = records.into_iter().skip(pag.offset).take(pag.limit).collect();

        Ok(QueryResult { records, total, offset: pag.offset, limit: pag.limit })
    }
}

pub fn has_include_deleted(f: &Filter) -> bool {
    match f {
        Filter::IncludeDeleted  => true,
        Filter::And(fs) | Filter::Or(fs) => fs.iter().any(has_include_deleted),
        Filter::Not(inner)      => has_include_deleted(inner),
        _                       => false,
    }
}

pub fn apply_filter(record: &Record, filter: &Filter) -> bool {
    match filter {
        Filter::IncludeDeleted => true,
        Filter::Eq(field, val) => {
            record.fields.get(field).map(|v| v == val).unwrap_or(false)
        }
        Filter::Neq(field, val) => {
            record.fields.get(field).map(|v| v != val).unwrap_or(true)
        }
        Filter::Gt(field, val) => {
            record.fields.get(field).map(|v| cmp_values(v, val) == std::cmp::Ordering::Greater).unwrap_or(false)
        }
        Filter::Lt(field, val) => {
            record.fields.get(field).map(|v| cmp_values(v, val) == std::cmp::Ordering::Less).unwrap_or(false)
        }
        Filter::Gte(field, val) => {
            record.fields.get(field).map(|v| cmp_values(v, val) != std::cmp::Ordering::Less).unwrap_or(false)
        }
        Filter::Lte(field, val) => {
            record.fields.get(field).map(|v| cmp_values(v, val) != std::cmp::Ordering::Greater).unwrap_or(false)
        }
        Filter::In(field, vals) => {
            record.fields.get(field).map(|v| vals.contains(v)).unwrap_or(false)
        }
        Filter::IsNull(field)    => record.fields.get(field).map(|v| *v == FieldValue::Null).unwrap_or(true),
        Filter::IsNotNull(field) => record.fields.get(field).map(|v| *v != FieldValue::Null).unwrap_or(false),
        Filter::And(fs) => fs.iter().all(|f| apply_filter(record, f)),
        Filter::Or(fs)  => fs.iter().any(|f| apply_filter(record, f)),
        Filter::Not(f)  => !apply_filter(record, f),
    }
}

fn cmp_values(a: &FieldValue, b: &FieldValue) -> std::cmp::Ordering {
    use FieldValue::*;
    match (a, b) {
        (Int(x), Int(y))           => x.cmp(y),
        (Float(x), Float(y))       => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Text(x), Text(y))         => x.cmp(y),
        (Timestamp(x), Timestamp(y)) => x.cmp(y),
        _                          => std::cmp::Ordering::Equal,
    }
}

fn compare_field(a: &Record, b: &Record, field: &str) -> std::cmp::Ordering {
    // Sort by system meta fields
    match field {
        "_id"         => a.meta.id.cmp(&b.meta.id),
        "_version"    => a.meta.version.cmp(&b.meta.version),
        "_created_at" => a.meta.created_at.cmp(&b.meta.created_at),
        "_updated_at" => a.meta.updated_at.cmp(&b.meta.updated_at),
        _ => {
            let av = a.fields.get(field);
            let bv = b.fields.get(field);
            match (av, bv) {
                (Some(x), Some(y)) => cmp_values(x, y),
                (Some(_), None)    => std::cmp::Ordering::Greater,
                (None, Some(_))    => std::cmp::Ordering::Less,
                (None, None)       => std::cmp::Ordering::Equal,
            }
        }
    }
}
