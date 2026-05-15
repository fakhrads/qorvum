use crate::certificate::PQCertificate;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

/// Thread-safe in-memory cert cache and CRL. Used by gateway for fast lookups.
pub struct IdentityStore {
    cache: Arc<RwLock<HashMap<[u8; 8], PQCertificate>>>,
    crl: Arc<RwLock<HashSet<[u8; 16]>>>,
}

impl Default for IdentityStore {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityStore {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            crl: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn add_cert(&self, cert: PQCertificate) {
        let fp = cert.fingerprint();
        self.cache.write().unwrap().insert(fp, cert);
    }

    pub fn get_by_fingerprint(&self, fp: &[u8; 8]) -> Option<PQCertificate> {
        self.cache.read().unwrap().get(fp).cloned()
    }

    pub fn add_to_crl(&self, serial: [u8; 16]) {
        self.crl.write().unwrap().insert(serial);
    }

    pub fn is_revoked(&self, serial: &[u8; 16]) -> bool {
        self.crl.read().unwrap().contains(serial)
    }
}
