use crate::certificate::{CertSubject, CertType, PQCertificate};
use crate::error::MspError;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use qorvum_crypto::{
    hash_many,
    signing::{PQKeypair, SigningAlgorithm},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::info;

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn days_to_nanos(days: u64) -> u64 {
    days * 24 * 3600 * 1_000_000_000
}

/// Exported public view of a CA — safe to distribute to peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaPublicInfo {
    pub ca_name: String,
    pub ca_org: String,
    pub ca_cert: PQCertificate,
    /// Serial (hex) → revocation reason
    pub crl: HashMap<String, String>,
    /// Fingerprint (hex) → issued certificate
    pub cert_registry: HashMap<String, PQCertificate>,
}

impl CaPublicInfo {
    pub fn is_revoked(&self, serial: &[u8; 16]) -> bool {
        self.crl.contains_key(&hex::encode(serial))
    }

    pub fn find_cert_by_fingerprint(&self, fp: &[u8; 8]) -> Option<&PQCertificate> {
        self.cert_registry.get(&hex::encode(fp))
    }
}

#[derive(Serialize, Deserialize)]
struct CaMeta {
    name: String,
    org: String,
    issued: Vec<String>, // serial hex strings
}

pub struct CertificateAuthority {
    pub name: String,
    pub org: String,
    keypair: PQKeypair,
    pub cert: PQCertificate,
    dir: PathBuf,
    issued: Vec<[u8; 16]>,
    revoked: HashSet<[u8; 16]>,
    /// serial hex → reason
    revoked_reasons: HashMap<String, String>,
}

impl CertificateAuthority {
    /// Initialize a new CA — generates keypair, self-signs CA cert, saves to `out_dir`.
    pub fn init(name: &str, org: &str, out_dir: &Path, passphrase: &str) -> Result<Self, MspError> {
        std::fs::create_dir_all(out_dir)?;
        std::fs::create_dir_all(out_dir.join("certs"))?;

        let keypair = PQKeypair::generate(SigningAlgorithm::Dilithium3)
            .map_err(|e| MspError::Crypto(e.to_string()))?;

        let not_before = now_nanos();
        let not_after = not_before + days_to_nanos(3650);

        let mut serial = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut serial);

        let subject = CertSubject {
            common_name: name.to_string(),
            org: org.to_string(),
            org_unit: Some("CA".to_string()),
            email: None,
        };

        // Build TBS manually, then sign
        let mut ca_cert = PQCertificate {
            serial,
            subject,
            issuer: name.to_string(),
            not_before,
            not_after,
            public_key: keypair.public_key().bytes.clone(),
            algorithm: SigningAlgorithm::Dilithium3,
            roles: vec!["CA".to_string()],
            cert_type: CertType::CA,
            extensions: std::collections::HashMap::new(),
            ca_signature: vec![],
            ca_public_key: keypair.public_key().bytes.clone(),
        };

        let tbs = ca_cert.tbs_bytes();
        let sig = keypair
            .sign(&tbs)
            .map_err(|e| MspError::Crypto(e.to_string()))?;
        ca_cert.ca_signature = sig.bytes;

        // Save encrypted key
        let key_path = out_dir.join("ca.key");
        save_encrypted_keypair(&keypair, passphrase, &key_path)?;

        // Save CA cert
        let cert_path = out_dir.join("ca.cert");
        std::fs::write(&cert_path, ca_cert.to_pem_like())?;

        // Save metadata
        let meta = CaMeta { name: name.to_string(), org: org.to_string(), issued: vec![] };
        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| MspError::Serialization(e.to_string()))?;
        std::fs::write(out_dir.join("ca.json"), meta_json)?;

        // Initialize empty CRL
        let crl_content = serde_json::json!({"revoked": {}});
        std::fs::write(out_dir.join("crl.json"), crl_content.to_string())?;

        info!("CA '{}' initialized at {:?}", name, out_dir);

        Ok(Self {
            name: name.to_string(),
            org: org.to_string(),
            keypair,
            cert: ca_cert,
            dir: out_dir.to_path_buf(),
            issued: vec![],
            revoked: HashSet::new(),
            revoked_reasons: HashMap::new(),
        })
    }

    /// Load an existing CA from disk.
    pub fn load(dir: &Path, passphrase: &str) -> Result<Self, MspError> {
        let cert_pem = std::fs::read_to_string(dir.join("ca.cert"))?;
        let cert = PQCertificate::from_pem_like(&cert_pem)?;

        let keypair = load_encrypted_keypair(passphrase, &dir.join("ca.key"))?;

        let meta: CaMeta = serde_json::from_str(
            &std::fs::read_to_string(dir.join("ca.json"))?
        )
        .map_err(|e| MspError::Serialization(e.to_string()))?;

        let issued: Vec<[u8; 16]> = meta
            .issued
            .iter()
            .filter_map(|s| {
                let bytes = hex::decode(s).ok()?;
                bytes.try_into().ok()
            })
            .collect();

        // Load CRL
        let (revoked, revoked_reasons) = load_crl(dir)?;

        Ok(Self {
            name: meta.name,
            org: meta.org,
            keypair,
            cert,
            dir: dir.to_path_buf(),
            issued,
            revoked,
            revoked_reasons,
        })
    }

    /// Issue a user certificate. Returns (cert, keypair) — caller persists the keypair.
    pub fn issue_user_cert(
        &mut self,
        subject: CertSubject,
        roles: Vec<String>,
        validity_days: u64,
    ) -> Result<(PQCertificate, PQKeypair), MspError> {
        self.issue_cert(subject, roles, validity_days, CertType::User)
    }

    /// Issue a node certificate.
    pub fn issue_node_cert(
        &mut self,
        node_name: &str,
        validity_days: u64,
    ) -> Result<(PQCertificate, PQKeypair), MspError> {
        let subject = CertSubject {
            common_name: node_name.to_string(),
            org: self.org.clone(),
            org_unit: Some("Node".to_string()),
            email: None,
        };
        self.issue_cert(subject, vec!["PEER_NODE".to_string()], validity_days, CertType::Node)
    }

    fn issue_cert(
        &mut self,
        subject: CertSubject,
        roles: Vec<String>,
        validity_days: u64,
        cert_type: CertType,
    ) -> Result<(PQCertificate, PQKeypair), MspError> {
        let user_kp = PQKeypair::generate(SigningAlgorithm::Dilithium3)
            .map_err(|e| MspError::Crypto(e.to_string()))?;

        let not_before = now_nanos();
        let not_after = not_before + days_to_nanos(validity_days);

        let mut serial = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut serial);

        let mut cert = PQCertificate {
            serial,
            subject,
            issuer: self.name.clone(),
            not_before,
            not_after,
            public_key: user_kp.public_key().bytes.clone(),
            algorithm: SigningAlgorithm::Dilithium3,
            roles,
            cert_type,
            extensions: std::collections::HashMap::new(),
            ca_signature: vec![],
            ca_public_key: self.keypair.public_key().bytes.clone(),
        };

        let tbs = cert.tbs_bytes();
        let sig = self
            .keypair
            .sign(&tbs)
            .map_err(|e| MspError::Crypto(e.to_string()))?;
        cert.ca_signature = sig.bytes;

        // Persist cert to certs/ directory (filename = common_name.cert)
        let name = cert.subject.common_name.clone();
        let cert_file = self.dir.join("certs").join(format!("{}.cert", name));
        std::fs::write(&cert_file, cert.to_pem_like())?;

        // Update metadata
        self.issued.push(serial);
        self.persist_meta()?;

        info!("Issued {} cert for '{}'", cert_type, name);
        Ok((cert, user_kp))
    }

    pub fn revoke(&mut self, serial: [u8; 16], reason: &str) -> Result<(), MspError> {
        let hex_serial = hex::encode(serial);
        self.revoked.insert(serial);
        self.revoked_reasons.insert(hex_serial, reason.to_string());
        self.persist_crl()
    }

    pub fn is_revoked(&self, serial: &[u8; 16]) -> bool {
        self.revoked.contains(serial)
    }

    pub fn export_public(&self) -> CaPublicInfo {
        // Build cert registry from certs/ directory
        let mut cert_registry = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(self.dir.join("certs")) {
            for entry in entries.flatten() {
                if let Ok(pem) = std::fs::read_to_string(entry.path()) {
                    if let Ok(cert) = PQCertificate::from_pem_like(&pem) {
                        cert_registry.insert(hex::encode(cert.fingerprint()), cert);
                    }
                }
            }
        }
        CaPublicInfo {
            ca_name: self.name.clone(),
            ca_org: self.org.clone(),
            ca_cert: self.cert.clone(),
            crl: self.revoked_reasons.clone(),
            cert_registry,
        }
    }

    fn persist_meta(&self) -> Result<(), MspError> {
        let meta = CaMeta {
            name: self.name.clone(),
            org: self.org.clone(),
            issued: self.issued.iter().map(hex::encode).collect(),
        };
        let json = serde_json::to_string_pretty(&meta)
            .map_err(|e| MspError::Serialization(e.to_string()))?;
        std::fs::write(self.dir.join("ca.json"), json)?;
        Ok(())
    }

    fn persist_crl(&self) -> Result<(), MspError> {
        let payload = serde_json::json!({ "revoked": self.revoked_reasons });
        std::fs::write(
            self.dir.join("crl.json"),
            serde_json::to_string_pretty(&payload)
                .map_err(|e| MspError::Serialization(e.to_string()))?,
        )?;
        Ok(())
    }
}

// ── Key file encryption (AES-256-GCM, passphrase-derived key via BLAKE3) ────

fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    hash_many(&[salt, passphrase.as_bytes()])
}

pub(crate) fn save_encrypted_keypair(
    kp: &PQKeypair,
    passphrase: &str,
    path: &Path,
) -> Result<(), MspError> {
    let alg_byte: u8 = 0; // Dilithium3
    let plaintext =
        bincode::serialize(&(alg_byte, kp.public_key().bytes.clone(), kp.secret_bytes()))
            .map_err(MspError::from)?;

    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key_bytes = derive_key(passphrase, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_slice())
        .map_err(|e| MspError::Encryption(e.to_string()))?;

    let mut file_bytes = Vec::with_capacity(16 + 12 + ciphertext.len());
    file_bytes.extend_from_slice(&salt);
    file_bytes.extend_from_slice(&nonce_bytes);
    file_bytes.extend_from_slice(&ciphertext);

    std::fs::write(path, file_bytes)?;
    Ok(())
}

pub(crate) fn load_encrypted_keypair(passphrase: &str, path: &Path) -> Result<PQKeypair, MspError> {
    let file_bytes = std::fs::read(path)?;
    if file_bytes.len() < 28 {
        return Err(MspError::WrongPassphrase);
    }

    let salt = &file_bytes[..16];
    let nonce_bytes = &file_bytes[16..28];
    let ciphertext = &file_bytes[28..];

    let key_bytes = derive_key(passphrase, salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| MspError::WrongPassphrase)?;

    let (alg_byte, pk_bytes, sk_bytes): (u8, Vec<u8>, Vec<u8>) =
        bincode::deserialize(&plaintext).map_err(MspError::from)?;

    let algorithm = if alg_byte == 0 {
        SigningAlgorithm::Dilithium3
    } else {
        SigningAlgorithm::Falcon512
    };

    Ok(PQKeypair::from_bytes(algorithm, pk_bytes, sk_bytes))
}

fn load_crl(dir: &Path) -> Result<(HashSet<[u8; 16]>, HashMap<String, String>), MspError> {
    let crl_path = dir.join("crl.json");
    if !crl_path.exists() {
        return Ok((HashSet::new(), HashMap::new()));
    }
    let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(crl_path)?)
        .map_err(|e| MspError::Serialization(e.to_string()))?;

    let mut revoked = HashSet::new();
    let mut reasons = HashMap::new();

    if let Some(map) = json["revoked"].as_object() {
        for (serial_hex, reason) in map {
            if let Ok(bytes) = hex::decode(serial_hex) {
                if let Ok(arr) = bytes.try_into() {
                    let arr: [u8; 16] = arr;
                    revoked.insert(arr);
                    reasons.insert(serial_hex.clone(), reason.as_str().unwrap_or("").to_string());
                }
            }
        }
    }
    Ok((revoked, reasons))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ca_init_and_load() {
        let dir = TempDir::new().unwrap();
        let ca = CertificateAuthority::init("Org1CA", "Org1", dir.path(), "secret123").unwrap();
        assert!(ca.cert.verify(), "CA cert should verify");

        let ca2 = CertificateAuthority::load(dir.path(), "secret123").unwrap();
        assert_eq!(ca.name, ca2.name);
        assert_eq!(ca.cert.serial, ca2.cert.serial);
        assert!(ca2.cert.verify());
    }

    #[test]
    fn test_ca_wrong_passphrase() {
        let dir = TempDir::new().unwrap();
        CertificateAuthority::init("Org1CA", "Org1", dir.path(), "correct").unwrap();
        let res = CertificateAuthority::load(dir.path(), "wrong");
        assert!(res.is_err());
    }

    #[test]
    fn test_issue_user_cert() {
        let dir = TempDir::new().unwrap();
        let mut ca = CertificateAuthority::init("Org1CA", "Org1", dir.path(), "pass").unwrap();
        let subject = CertSubject {
            common_name: "alice".to_string(),
            org: "Org1".to_string(),
            org_unit: None,
            email: Some("alice@org1.com".to_string()),
        };
        let (cert, _kp) = ca.issue_user_cert(subject, vec!["ADMIN".to_string()], 365).unwrap();
        assert!(cert.verify(), "User cert should verify against CA");
        assert_eq!(cert.roles, vec!["ADMIN"]);
    }

    #[test]
    fn test_revoke_and_check() {
        let dir = TempDir::new().unwrap();
        let mut ca = CertificateAuthority::init("Org1CA", "Org1", dir.path(), "pass").unwrap();
        let subject = CertSubject {
            common_name: "bob".to_string(),
            org: "Org1".to_string(),
            org_unit: None,
            email: None,
        };
        let (cert, _) = ca.issue_user_cert(subject, vec![], 365).unwrap();
        assert!(!ca.is_revoked(&cert.serial));
        ca.revoke(cert.serial, "Key compromise").unwrap();
        assert!(ca.is_revoked(&cert.serial));

        // Verify CRL persisted
        let ca2 = CertificateAuthority::load(dir.path(), "pass").unwrap();
        assert!(ca2.is_revoked(&cert.serial));
    }
}
