use crate::ca::CaPublicInfo;
use crate::certificate::PQCertificate;
use crate::error::MspError;
use crate::identity::Identity;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine;
use qorvum_crypto::signing::{verify, PublicKey, Signature, SigningAlgorithm};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub subject: String,
    pub org: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QorvumToken {
    pub cert_fingerprint: [u8; 8],
    pub issued_at: u64,
    pub expires_at: u64,
    pub nonce: [u8; 16],
    pub claims: TokenClaims,
    pub signature: Vec<u8>,
}

// TBS fields for deterministic signing
#[derive(Serialize)]
struct TokenTbs<'a> {
    cert_fingerprint: &'a [u8; 8],
    issued_at: u64,
    expires_at: u64,
    nonce: &'a [u8; 16],
    claims: &'a TokenClaims,
}

impl QorvumToken {
    pub fn issue(identity: &Identity, ttl_secs: u64) -> Result<Self, MspError> {
        let issued_at = now_nanos();
        let expires_at = issued_at + ttl_secs * 1_000_000_000;
        let cert_fingerprint = identity.cert.fingerprint();

        let mut nonce = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce);

        let claims = TokenClaims {
            subject: identity.cert.subject.common_name.clone(),
            org: identity.cert.subject.org.clone(),
            roles: identity.cert.roles.clone(),
        };

        let tbs = TokenTbs {
            cert_fingerprint: &cert_fingerprint,
            issued_at,
            expires_at,
            nonce: &nonce,
            claims: &claims,
        };
        let tbs_bytes = bincode::serialize(&tbs)
            .map_err(|e| MspError::Serialization(e.to_string()))?;

        let sig = identity.sign(&tbs_bytes)?;

        Ok(Self {
            cert_fingerprint,
            issued_at,
            expires_at,
            nonce,
            claims,
            signature: sig.bytes,
        })
    }

    pub fn verify_with_ca(&self, ca_public: &CaPublicInfo) -> Result<TokenClaims, MspError> {
        let now = now_nanos();
        if now > self.expires_at {
            return Err(MspError::TokenExpired);
        }

        let cert = ca_public
            .find_cert_by_fingerprint(&self.cert_fingerprint)
            .ok_or_else(|| MspError::UnknownCertFingerprint(hex::encode(self.cert_fingerprint)))?;

        self.verify_signature(cert)?;

        if ca_public.is_revoked(&cert.serial) {
            return Err(MspError::CertRevoked(hex::encode(cert.serial)));
        }

        Ok(self.claims.clone())
    }

    fn verify_signature(&self, cert: &PQCertificate) -> Result<(), MspError> {
        let tbs = TokenTbs {
            cert_fingerprint: &self.cert_fingerprint,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            nonce: &self.nonce,
            claims: &self.claims,
        };
        let tbs_bytes = bincode::serialize(&tbs)
            .map_err(|e| MspError::Serialization(e.to_string()))?;

        let pk = PublicKey {
            algorithm: cert.algorithm,
            bytes: cert.public_key.clone(),
        };
        let sig = Signature {
            algorithm: SigningAlgorithm::Dilithium3,
            bytes: self.signature.clone(),
        };

        if !verify(&pk, &tbs_bytes, &sig) {
            return Err(MspError::TokenInvalidSignature);
        }
        Ok(())
    }

    pub fn to_bearer_string(&self) -> Result<String, MspError> {
        let bytes = bincode::serialize(self)
            .map_err(|e| MspError::Serialization(e.to_string()))?;
        Ok(B64URL.encode(&bytes))
    }

    pub fn from_bearer_string(s: &str) -> Result<Self, MspError> {
        let bytes = B64URL.decode(s).map_err(|_| MspError::InvalidPem)?;
        bincode::deserialize(&bytes).map_err(MspError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CertificateAuthority;
    use crate::certificate::CertSubject;
    use crate::identity::Identity;
    use tempfile::TempDir;

    fn setup() -> (TempDir, CaPublicInfo, Identity) {
        let dir = TempDir::new().unwrap();
        let mut ca =
            CertificateAuthority::init("TestCA", "Org1", dir.path(), "pass").unwrap();
        let subject = CertSubject {
            common_name: "alice".to_string(),
            org: "Org1".to_string(),
            org_unit: None,
            email: None,
        };
        let (cert, kp) = ca.issue_user_cert(subject, vec!["ADMIN".to_string()], 365).unwrap();
        let ca_pub = ca.export_public();
        let identity = Identity::new(cert, kp);
        (dir, ca_pub, identity)
    }

    #[test]
    fn test_issue_and_verify_token() {
        let (_dir, ca_pub, identity) = setup();
        let token = QorvumToken::issue(&identity, 3600).unwrap();
        let claims = token.verify_with_ca(&ca_pub).unwrap();
        assert_eq!(claims.subject, "alice");
        assert_eq!(claims.roles, vec!["ADMIN"]);
    }

    #[test]
    fn test_token_expired() {
        let (_dir, ca_pub, identity) = setup();
        let mut token = QorvumToken::issue(&identity, 3600).unwrap();
        token.expires_at = 1; // force to ancient past — expiry check runs before sig check
        let res = token.verify_with_ca(&ca_pub);
        assert!(matches!(res, Err(MspError::TokenExpired)));
    }

    #[test]
    fn test_token_replay_different_nonce() {
        let (_dir, _, identity) = setup();
        let t1 = QorvumToken::issue(&identity, 3600).unwrap();
        let t2 = QorvumToken::issue(&identity, 3600).unwrap();
        assert_ne!(t1.nonce, t2.nonce, "Each token must have a unique nonce");
    }

    #[test]
    fn test_token_tampered_signature() {
        let (_dir, ca_pub, identity) = setup();
        let mut token = QorvumToken::issue(&identity, 3600).unwrap();
        token.claims.subject = "eve".to_string(); // tamper claims
        let res = token.verify_with_ca(&ca_pub);
        assert!(matches!(res, Err(MspError::TokenInvalidSignature)));
    }

    #[test]
    fn test_bearer_string_roundtrip() {
        let (_dir, ca_pub, identity) = setup();
        let token = QorvumToken::issue(&identity, 3600).unwrap();
        let bearer = token.to_bearer_string().unwrap();
        let token2 = QorvumToken::from_bearer_string(&bearer).unwrap();
        let claims = token2.verify_with_ca(&ca_pub).unwrap();
        assert_eq!(claims.subject, "alice");
    }
}
