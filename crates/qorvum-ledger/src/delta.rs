//! Delta encoding for record history.
//!
//! Instead of full snapshots per version, only the changed fields are stored.
//! `delta_store` key: `"{collection}~{id}~{version:016x}"`

use crate::error::LedgerError;
use crate::record::{FieldValue, Record, RecordMeta};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What happened to a single field between two versions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FieldDelta {
    /// Field was added or its value changed.
    Set(FieldValue),
    /// Field was removed from the record.
    Removed,
}

/// Compact diff between two consecutive record versions.
///
/// `delta_hash` is BLAKE3 over all other fields (sorted, deterministic).
/// Verify with [`RecordDelta::apply`] before trusting reconstructed state.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordDelta {
    pub collection:   String,
    pub partition:    String,
    pub id:           String,
    /// Version before this change (0 = first insert).
    pub from_version: u64,
    /// Version after this change.
    pub to_version:   u64,
    pub tx_id:        [u8; 32],
    pub block_num:    u64,
    /// `updated_at` from the new record (Unix nanos).
    pub timestamp:    u64,
    pub is_deleted:   bool,
    pub field_deltas: HashMap<String, FieldDelta>,
    /// BLAKE3 of all fields above — tamper evidence.
    pub delta_hash:   [u8; 32],
}

impl RecordDelta {
    // ── Public API ────────────────────────────────────────────────────────────

    /// Compute the delta between an old and new record version.
    ///
    /// `old = None` means this is the first insert (from_version = 0).
    /// Only changed or removed fields are stored; unchanged fields are omitted.
    pub fn compute(old: Option<&Record>, new: &Record) -> Self {
        let empty: HashMap<String, FieldValue> = HashMap::new();
        let old_fields = old.map(|r| &r.fields).unwrap_or(&empty);

        let mut field_deltas: HashMap<String, FieldDelta> = HashMap::new();

        // Fields that are new or changed.
        for (key, new_val) in &new.fields {
            match old_fields.get(key) {
                Some(old_val) if old_val == new_val => {} // unchanged
                _ => {
                    field_deltas.insert(key.clone(), FieldDelta::Set(new_val.clone()));
                }
            }
        }

        // Fields that were removed.
        for key in old_fields.keys() {
            if !new.fields.contains_key(key) {
                field_deltas.insert(key.clone(), FieldDelta::Removed);
            }
        }

        let from_version = old.map(|r| r.meta.version).unwrap_or(0);
        let mut delta = RecordDelta {
            collection:   new.meta.collection.clone(),
            partition:    new.meta.partition.clone(),
            id:           new.meta.id.clone(),
            from_version,
            to_version:   new.meta.version,
            tx_id:        new.meta.tx_id,
            block_num:    new.meta.block_num,
            timestamp:    new.meta.updated_at,
            is_deleted:   new.meta.is_deleted,
            field_deltas,
            delta_hash:   [0u8; 32],
        };

        delta.delta_hash = delta.compute_hash();
        delta
    }

    /// Reconstruct a `Record` by applying this delta on top of a base state.
    ///
    /// Returns `Err(LedgerError::HashMismatch)` if the delta was tampered with.
    /// `base = None` is valid for the first insert (from_version = 0).
    pub fn apply(base: Option<Record>, delta: &RecordDelta) -> Result<Record, LedgerError> {
        // Integrity check first.
        let expected = delta.compute_hash();
        if expected != delta.delta_hash {
            return Err(LedgerError::HashMismatch);
        }

        let mut record = match base {
            Some(r) => r,
            None => Record {
                meta: RecordMeta {
                    id:            delta.id.clone(),
                    collection:    delta.collection.clone(),
                    partition:     delta.partition.clone(),
                    version:       delta.to_version,
                    created_at:    delta.timestamp,
                    updated_at:    delta.timestamp,
                    created_by:    String::new(),
                    updated_by:    String::new(),
                    is_deleted:    delta.is_deleted,
                    delete_reason: None,
                    tx_id:         delta.tx_id,
                    block_num:     delta.block_num,
                },
                fields: HashMap::new(),
            },
        };

        // Update mutable metadata.
        record.meta.version    = delta.to_version;
        record.meta.updated_at = delta.timestamp;
        record.meta.is_deleted = delta.is_deleted;
        record.meta.tx_id      = delta.tx_id;
        record.meta.block_num  = delta.block_num;

        // Apply field changes.
        for (key, field_delta) in &delta.field_deltas {
            match field_delta {
                FieldDelta::Set(val) => { record.fields.insert(key.clone(), val.clone()); }
                FieldDelta::Removed  => { record.fields.remove(key); }
            }
        }

        Ok(record)
    }

    /// Storage key for this delta: `"{collection}~{id}~{to_version:016x}"`.
    pub fn storage_key(&self) -> String {
        delta_key(&self.collection, &self.id, self.to_version)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    /// BLAKE3 over all fields except `delta_hash` itself.
    /// Field deltas are sorted by key for determinism regardless of HashMap order.
    fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Scalar fields — length-prefixed to prevent collisions.
        Self::hash_str(&mut hasher, &self.collection);
        Self::hash_str(&mut hasher, &self.partition);
        Self::hash_str(&mut hasher, &self.id);
        hasher.update(&self.from_version.to_le_bytes());
        hasher.update(&self.to_version.to_le_bytes());
        hasher.update(&self.tx_id);
        hasher.update(&self.block_num.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&[self.is_deleted as u8]);

        // Sort field deltas for determinism.
        let mut sorted_keys: Vec<&String> = self.field_deltas.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            Self::hash_str(&mut hasher, key);
            let val_bytes = serde_json::to_vec(&self.field_deltas[key])
                .expect("FieldDelta serialization is infallible");
            hasher.update(&(val_bytes.len() as u64).to_le_bytes());
            hasher.update(&val_bytes);
        }

        *hasher.finalize().as_bytes()
    }

    #[inline]
    fn hash_str(hasher: &mut blake3::Hasher, s: &str) {
        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());
    }
}

/// Storage key: `"{collection}~{id}~{version:016x}"`.
/// 16 hex digits cover the full u64 range and sort lexicographically.
pub fn delta_key(collection: &str, id: &str, version: u64) -> String {
    format!("{}~{}~{:016x}", collection, id, version)
}

/// Prefix for scanning all deltas of one record.
pub fn delta_prefix(collection: &str, id: &str) -> String {
    format!("{}~{}~", collection, id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::RecordMeta;

    fn meta(id: &str, version: u64) -> RecordMeta {
        RecordMeta {
            id:            id.to_string(),
            collection:    "employees".to_string(),
            partition:     "HR".to_string(),
            version,
            created_at:    1_000_000,
            updated_at:    1_000_000 + version * 1_000,
            created_by:    "alice".to_string(),
            updated_by:    "alice".to_string(),
            is_deleted:    false,
            delete_reason: None,
            tx_id:         [version as u8; 32],
            block_num:     version,
        }
    }

    fn record_v1() -> Record {
        let mut fields = HashMap::new();
        fields.insert("name".into(), FieldValue::Text("Alice".into()));
        fields.insert("age".into(),  FieldValue::Int(30));
        fields.insert("dept".into(), FieldValue::Text("Engineering".into()));
        Record { meta: meta("EMP001", 1), fields }
    }

    fn record_v2() -> Record {
        let mut fields = HashMap::new();
        fields.insert("name".into(), FieldValue::Text("Alice".into())); // unchanged
        fields.insert("age".into(),  FieldValue::Int(31));              // changed
        fields.insert("role".into(), FieldValue::Text("Lead".into()));  // added
        // "dept" removed
        Record { meta: meta("EMP001", 2), fields }
    }

    fn record_v3() -> Record {
        let mut fields = HashMap::new();
        fields.insert("name".into(), FieldValue::Text("Alice".into()));
        fields.insert("age".into(),  FieldValue::Int(31));
        fields.insert("role".into(), FieldValue::Text("Director".into())); // changed
        Record { meta: meta("EMP001", 3), fields }
    }

    #[test]
    fn compute_first_insert() {
        let r = record_v1();
        let delta = RecordDelta::compute(None, &r);

        assert_eq!(delta.from_version, 0);
        assert_eq!(delta.to_version, 1);
        // All fields are Set on first insert.
        assert!(matches!(delta.field_deltas["name"], FieldDelta::Set(FieldValue::Text(_))));
        assert!(matches!(delta.field_deltas["age"],  FieldDelta::Set(FieldValue::Int(30))));
        assert!(matches!(delta.field_deltas["dept"], FieldDelta::Set(FieldValue::Text(_))));
        assert_ne!(delta.delta_hash, [0u8; 32]);
    }

    #[test]
    fn compute_update_detects_changes() {
        let v1 = record_v1();
        let v2 = record_v2();
        let delta = RecordDelta::compute(Some(&v1), &v2);

        assert_eq!(delta.from_version, 1);
        assert_eq!(delta.to_version, 2);

        // "name" unchanged — not in delta.
        assert!(!delta.field_deltas.contains_key("name"));

        // "age" changed 30 → 31.
        assert!(matches!(delta.field_deltas["age"], FieldDelta::Set(FieldValue::Int(31))));

        // "role" added.
        assert!(matches!(delta.field_deltas["role"], FieldDelta::Set(FieldValue::Text(_))));

        // "dept" removed.
        assert_eq!(delta.field_deltas["dept"], FieldDelta::Removed);
    }

    #[test]
    fn apply_first_insert() {
        let r = record_v1();
        let delta = RecordDelta::compute(None, &r);
        let reconstructed = RecordDelta::apply(None, &delta).unwrap();

        assert_eq!(reconstructed.meta.id, "EMP001");
        assert_eq!(reconstructed.meta.version, 1);
        assert_eq!(reconstructed.fields["name"], FieldValue::Text("Alice".into()));
        assert_eq!(reconstructed.fields["age"],  FieldValue::Int(30));
        assert_eq!(reconstructed.fields["dept"], FieldValue::Text("Engineering".into()));
    }

    #[test]
    fn apply_update_produces_correct_record() {
        let v1 = record_v1();
        let v2 = record_v2();

        let d1 = RecordDelta::compute(None,      &v1);
        let d2 = RecordDelta::compute(Some(&v1), &v2);

        let r1 = RecordDelta::apply(None, &d1).unwrap();
        let r2 = RecordDelta::apply(Some(r1), &d2).unwrap();

        assert_eq!(r2.meta.version, 2);
        assert_eq!(r2.fields["name"], FieldValue::Text("Alice".into())); // preserved
        assert_eq!(r2.fields["age"],  FieldValue::Int(31));              // updated
        assert_eq!(r2.fields["role"], FieldValue::Text("Lead".into()));  // added
        assert!(!r2.fields.contains_key("dept"));                        // removed
    }

    #[test]
    fn hash_mismatch_detected() {
        let r = record_v1();
        let mut delta = RecordDelta::compute(None, &r);

        // Tamper with a field after hash was computed.
        delta.block_num = 9999;

        let err = RecordDelta::apply(None, &delta).unwrap_err();
        assert!(matches!(err, LedgerError::HashMismatch));
    }

    #[test]
    fn hash_mismatch_on_field_delta_tamper() {
        let v1 = record_v1();
        let v2 = record_v2();
        let mut delta = RecordDelta::compute(Some(&v1), &v2);

        // Inject a fake field change.
        delta.field_deltas.insert("salary".into(), FieldDelta::Set(FieldValue::Int(999_999)));

        let err = RecordDelta::apply(Some(v1), &delta).unwrap_err();
        assert!(matches!(err, LedgerError::HashMismatch));
    }

    #[test]
    fn round_trip_reconstruct_version_2() {
        let v1 = record_v1();
        let v2 = record_v2();
        let v3 = record_v3();

        let d1 = RecordDelta::compute(None,      &v1);
        let d2 = RecordDelta::compute(Some(&v1), &v2);
        let d3 = RecordDelta::compute(Some(&v2), &v3);

        // Reconstruct at version 2 from the chain [d1, d2, d3].
        let deltas = vec![d1, d2, d3];

        let mut record: Option<Record> = None;
        for delta in deltas.iter().filter(|d| d.to_version <= 2) {
            record = Some(RecordDelta::apply(record, delta).unwrap());
        }
        let at_v2 = record.unwrap();

        // Must match v2 snapshot exactly.
        assert_eq!(at_v2.meta.version, 2);
        assert_eq!(at_v2.fields["age"],  FieldValue::Int(31));
        assert_eq!(at_v2.fields["role"], FieldValue::Text("Lead".into()));
        assert!(!at_v2.fields.contains_key("dept"));

        // And NOT have v3 changes.
        assert_ne!(at_v2.fields.get("role"), Some(&FieldValue::Text("Director".into())));
    }

    #[test]
    fn storage_key_is_lexicographically_ordered() {
        let k1 = delta_key("col", "id", 1);
        let k2 = delta_key("col", "id", 2);
        let k3 = delta_key("col", "id", 256);
        assert!(k1 < k2);
        assert!(k2 < k3);
    }

    #[test]
    fn delta_store_integration_with_memory_store() {
        use crate::backends::MemoryStore;
        use crate::store::LedgerStore;

        let store = MemoryStore::new();

        let v1 = record_v1();
        let v2 = record_v2();
        let v3 = record_v3();

        let d1 = RecordDelta::compute(None,      &v1);
        let d2 = RecordDelta::compute(Some(&v1), &v2);
        let d3 = RecordDelta::compute(Some(&v2), &v3);

        store.put_delta(&d1).unwrap();
        store.put_delta(&d2).unwrap();
        store.put_delta(&d3).unwrap();

        // get_deltas returns all in order.
        let deltas = store.get_deltas("employees", "EMP001").unwrap();
        assert_eq!(deltas.len(), 3);
        assert_eq!(deltas[0].to_version, 1);
        assert_eq!(deltas[1].to_version, 2);
        assert_eq!(deltas[2].to_version, 3);

        // reconstruct_at_version(2) matches v2 snapshot.
        let at_v2 = store
            .reconstruct_at_version("employees", "HR", "EMP001", 2)
            .unwrap()
            .unwrap();
        assert_eq!(at_v2.meta.version, 2);
        assert_eq!(at_v2.fields["age"], FieldValue::Int(31));
        assert!(!at_v2.fields.contains_key("dept"));
    }
}
