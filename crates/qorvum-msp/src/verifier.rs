use crate::ca::CaPublicInfo;
use crate::certificate::{CertType, PQCertificate};
use crate::error::MspError;
use crate::token::QorvumToken;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedIdentity {
    pub subject: String,
    pub org: String,
    pub roles: Vec<String>,
    pub cert_type: CertType,
}

pub struct IdentityVerifier {
    trusted_cas: Vec<CaPublicInfo>,
}

impl IdentityVerifier {
    /// Load all CAs from the given directories (one CA per directory).
    pub fn new(ca_dirs: &[PathBuf]) -> Result<Self, MspError> {
        let mut trusted_cas = Vec::new();
        for dir in ca_dirs {
            let cert_path = dir.join("ca.cert");
            if !cert_path.exists() {
                tracing::warn!("No ca.cert found in {:?}, skipping", dir);
                continue;
            }
            let ca_pub = load_ca_public_info(dir)?;
            tracing::info!("Loaded CA '{}' from {:?}", ca_pub.ca_name, dir);
            trusted_cas.push(ca_pub);
        }
        Ok(Self { trusted_cas })
    }

    pub fn is_configured(&self) -> bool {
        !self.trusted_cas.is_empty()
    }

    /// Full verification pipeline: parse bearer → sig verify → expiry → revocation.
    pub fn verify_token(&self, bearer: &str) -> Result<VerifiedIdentity, MspError> {
        let token = QorvumToken::from_bearer_string(bearer)?;

        // Find which CA issued the cert for this token
        for ca in &self.trusted_cas {
            if let Some(cert) = ca.find_cert_by_fingerprint(&token.cert_fingerprint) {
                let claims = token.verify_with_ca(ca)?;
                return Ok(VerifiedIdentity {
                    subject: claims.subject,
                    org: claims.org,
                    roles: claims.roles,
                    cert_type: cert.cert_type,
                });
            }
        }
        Err(MspError::UnknownCertFingerprint(hex::encode(
            token.cert_fingerprint,
        )))
    }

    /// Add a freshly-issued cert to the in-memory registry of the first trusted CA.
    /// Called after enrollment so new users can immediately authenticate without a restart.
    pub fn add_cert(&mut self, cert: PQCertificate) {
        if let Some(ca) = self.trusted_cas.first_mut() {
            ca.cert_registry.insert(hex::encode(cert.fingerprint()), cert);
        }
    }

    /// Record a revocation in the in-memory CRL so that existing tokens for the
    /// revoked user are rejected immediately (without a restart).
    pub fn add_revocation(&mut self, serial: [u8; 16], reason: String) {
        if let Some(ca) = self.trusted_cas.first_mut() {
            ca.crl.insert(hex::encode(serial), reason);
        }
    }

    /// Return all trusted CA public info — used by the federation certs endpoint.
    pub fn trusted_cas(&self) -> &[CaPublicInfo] {
        &self.trusted_cas
    }

    /// Verify a certificate against all trusted CAs.
    pub fn verify_cert(&self, cert: &PQCertificate) -> Result<(), MspError> {
        if !cert.verify() {
            return Err(MspError::CertVerificationFailed(
                "CA signature check failed".to_string(),
            ));
        }

        for ca in &self.trusted_cas {
            if ca.is_revoked(&cert.serial) {
                return Err(MspError::CertRevoked(hex::encode(cert.serial)));
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        if !cert.is_valid_at(now) {
            return Err(MspError::CertExpired);
        }
        Ok(())
    }
}

fn load_ca_public_info(dir: &std::path::Path) -> Result<CaPublicInfo, MspError> {
    use crate::certificate::PQCertificate;
    use std::collections::HashMap;

    let cert_pem = std::fs::read_to_string(dir.join("ca.cert"))?;
    let ca_cert = PQCertificate::from_pem_like(&cert_pem)?;

    // Load CRL
    let crl_path = dir.join("crl.json");
    let crl: HashMap<String, String> = if crl_path.exists() {
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(crl_path)?)
                .map_err(|e| MspError::Serialization(e.to_string()))?;
        json["revoked"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Load cert registry from certs/ and users/ directories.
    // UserStore.enroll() saves to users/, so we must scan both on startup
    // to avoid "unknown certificate fingerprint" after a node restart.
    let mut cert_registry = HashMap::new();
    for subdir in &["certs", "users"] {
        let scan_dir = dir.join(subdir);
        if scan_dir.exists() {
            for entry in std::fs::read_dir(&scan_dir)?.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "cert") {
                    if let Ok(pem) = std::fs::read_to_string(&path) {
                        if let Ok(cert) = PQCertificate::from_pem_like(&pem) {
                            cert_registry.insert(hex::encode(cert.fingerprint()), cert);
                        }
                    }
                }
            }
        }
    }

    // Load CA name/org from ca.json if available
    let (ca_name, ca_org) = {
        let meta_path = dir.join("ca.json");
        if meta_path.exists() {
            let meta: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(meta_path)?)
                    .map_err(|e| MspError::Serialization(e.to_string()))?;
            (
                meta["name"].as_str().unwrap_or("").to_string(),
                meta["org"].as_str().unwrap_or("").to_string(),
            )
        } else {
            (
                ca_cert.subject.common_name.clone(),
                ca_cert.subject.org.clone(),
            )
        }
    };

    Ok(CaPublicInfo {
        ca_name,
        ca_org,
        ca_cert,
        crl,
        cert_registry,
    })
}
