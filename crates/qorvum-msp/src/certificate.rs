use crate::error::MspError;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use qorvum_crypto::{hash_many, signing::SigningAlgorithm, signing::verify, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

const PEM_HEADER: &str = "-----BEGIN QORVUM CERTIFICATE-----";
const PEM_FOOTER: &str = "-----END QORVUM CERTIFICATE-----";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CertSubject {
    pub common_name: String,
    pub org: String,
    pub org_unit: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CertType {
    User,
    Node,
    CA,
}

impl std::fmt::Display for CertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertType::User => write!(f, "User"),
            CertType::Node => write!(f, "Node"),
            CertType::CA => write!(f, "CA"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PQCertificate {
    pub serial: [u8; 16],
    pub subject: CertSubject,
    pub issuer: String,
    pub not_before: u64,
    pub not_after: u64,
    pub public_key: Vec<u8>,
    pub algorithm: SigningAlgorithm,
    pub roles: Vec<String>,
    pub cert_type: CertType,
    pub extensions: HashMap<String, String>,
    pub ca_signature: Vec<u8>,
    pub ca_public_key: Vec<u8>,
}

// Internal struct for deterministic TBS serialization (BTreeMap for sorted keys)
#[derive(Serialize)]
struct TbsCert<'a> {
    serial: &'a [u8; 16],
    subject: &'a CertSubject,
    issuer: &'a str,
    not_before: u64,
    not_after: u64,
    public_key: &'a [u8],
    algorithm: SigningAlgorithm,
    roles: &'a [String],
    cert_type: &'a CertType,
    extensions: BTreeMap<&'a str, &'a str>,
}

impl PQCertificate {
    pub fn tbs_bytes(&self) -> Vec<u8> {
        let sorted_ext: BTreeMap<&str, &str> = self
            .extensions
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let tbs = TbsCert {
            serial: &self.serial,
            subject: &self.subject,
            issuer: &self.issuer,
            not_before: self.not_before,
            not_after: self.not_after,
            public_key: &self.public_key,
            algorithm: self.algorithm,
            roles: &self.roles,
            cert_type: &self.cert_type,
            extensions: sorted_ext,
        };
        bincode::serialize(&tbs).unwrap_or_default()
    }

    pub fn verify(&self) -> bool {
        let ca_pk = PublicKey {
            algorithm: self.algorithm,
            bytes: self.ca_public_key.clone(),
        };
        let sig = Signature {
            algorithm: self.algorithm,
            bytes: self.ca_signature.clone(),
        };
        verify(&ca_pk, &self.tbs_bytes(), &sig)
    }

    pub fn is_valid_at(&self, ts: u64) -> bool {
        ts >= self.not_before && ts <= self.not_after
    }

    pub fn fingerprint(&self) -> [u8; 8] {
        let h = hash_many(&[&self.tbs_bytes()]);
        h[..8].try_into().unwrap()
    }

    pub fn to_pem_like(&self) -> String {
        let bytes = bincode::serialize(self).unwrap_or_default();
        format!(
            "{}\n{}\n{}",
            PEM_HEADER,
            B64.encode(&bytes),
            PEM_FOOTER
        )
    }

    pub fn from_pem_like(s: &str) -> Result<Self, MspError> {
        let s = s.trim();
        let body = s
            .strip_prefix(PEM_HEADER)
            .and_then(|s| s.trim().strip_suffix(PEM_FOOTER))
            .ok_or(MspError::InvalidPem)?
            .trim();
        let bytes = B64.decode(body).map_err(|_| MspError::InvalidPem)?;
        bincode::deserialize(&bytes).map_err(MspError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CertificateAuthority;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    fn now_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn make_ca(dir: &TempDir) -> CertificateAuthority {
        CertificateAuthority::init("TestCA", "TestOrg", dir.path(), "testpass").unwrap()
    }

    #[test]
    fn test_cert_self_sign_and_verify() {
        let dir = TempDir::new().unwrap();
        let ca = make_ca(&dir);
        assert!(ca.cert.verify(), "CA self-signed cert should verify");
    }

    #[test]
    fn test_cert_invalid_signature() {
        let dir = TempDir::new().unwrap();
        let mut ca = make_ca(&dir);
        ca.cert.ca_signature[0] ^= 0xFF;
        assert!(!ca.cert.verify(), "Tampered cert should fail verification");
    }

    #[test]
    fn test_cert_expired() {
        let dir = TempDir::new().unwrap();
        let ca = make_ca(&dir);
        // CA cert expires in 3650 days from now — past timestamp should fail
        assert!(!ca.cert.is_valid_at(1_000_000), "Ancient timestamp should be invalid");
        assert!(ca.cert.is_valid_at(now_nanos()), "Current timestamp should be valid");
    }

    #[test]
    fn test_pem_roundtrip() {
        let dir = TempDir::new().unwrap();
        let ca = make_ca(&dir);
        let pem = ca.cert.to_pem_like();
        let loaded = PQCertificate::from_pem_like(&pem).unwrap();
        assert_eq!(
            ca.cert.serial, loaded.serial,
            "Serial should survive PEM roundtrip"
        );
        assert_eq!(ca.cert.subject.common_name, loaded.subject.common_name);
        assert!(loaded.verify(), "Loaded cert should still verify");
    }
}
