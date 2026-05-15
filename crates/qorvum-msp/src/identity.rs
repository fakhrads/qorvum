use crate::ca::{load_encrypted_keypair, save_encrypted_keypair};
use crate::certificate::PQCertificate;
use crate::error::MspError;
use qorvum_crypto::signing::{PQKeypair, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub struct Identity {
    pub cert: PQCertificate,
    keypair: PQKeypair,
}

impl Identity {
    pub fn new(cert: PQCertificate, keypair: PQKeypair) -> Self {
        Self { cert, keypair }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProfile {
    pub name: String,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl Identity {
    /// Load identity from cert + encrypted key files.
    pub fn load(cert_path: &Path, key_path: &Path, passphrase: &str) -> Result<Self, MspError> {
        let pem = std::fs::read_to_string(cert_path)?;
        let cert = PQCertificate::from_pem_like(&pem)?;
        let keypair = load_encrypted_keypair(passphrase, key_path)?;
        Ok(Self { cert, keypair })
    }

    /// Load identity from cert + unencrypted (raw bincode) key file.
    pub fn load_unencrypted(cert_path: &Path, key_path: &Path) -> Result<Self, MspError> {
        let pem = std::fs::read_to_string(cert_path)?;
        let cert = PQCertificate::from_pem_like(&pem)?;
        let key_bytes = std::fs::read(key_path)?;
        let (alg_byte, pk_bytes, sk_bytes): (u8, Vec<u8>, Vec<u8>) =
            bincode::deserialize(&key_bytes).map_err(MspError::from)?;
        let algorithm = if alg_byte == 0 {
            qorvum_crypto::signing::SigningAlgorithm::Dilithium3
        } else {
            qorvum_crypto::signing::SigningAlgorithm::Falcon512
        };
        let keypair = PQKeypair::from_bytes(algorithm, pk_bytes, sk_bytes);
        Ok(Self { cert, keypair })
    }

    /// Save cert as `{name}.cert` and encrypted key as `{name}.key` in `dir`.
    pub fn save(&self, dir: &Path, passphrase: &str) -> Result<IdentityProfile, MspError> {
        std::fs::create_dir_all(dir)?;
        let name = self.cert.subject.common_name.clone();
        let cert_path = dir.join(format!("{}.cert", name));
        let key_path = dir.join(format!("{}.key", name));
        std::fs::write(&cert_path, self.cert.to_pem_like())?;
        save_encrypted_keypair(&self.keypair, passphrase, &key_path)?;
        Ok(IdentityProfile { name, cert_path, key_path })
    }

    /// Save cert as `{name}.cert` and unencrypted key as `{name}.key` (for dev use).
    pub fn save_unencrypted(&self, dir: &Path) -> Result<IdentityProfile, MspError> {
        std::fs::create_dir_all(dir)?;
        let name = self.cert.subject.common_name.clone();
        let cert_path = dir.join(format!("{}.cert", name));
        let key_path = dir.join(format!("{}.key", name));
        std::fs::write(&cert_path, self.cert.to_pem_like())?;
        // Store as (alg_byte, pk_bytes, sk_bytes) — consistent with node validator format
        let alg_byte: u8 = 0;
        let raw = bincode::serialize(&(
            alg_byte,
            self.keypair.public_key().bytes.clone(),
            self.keypair.secret_bytes(),
        ))
        .map_err(MspError::from)?;
        std::fs::write(&key_path, raw)?;
        Ok(IdentityProfile { name, cert_path, key_path })
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature, MspError> {
        self.keypair
            .sign(message)
            .map_err(|e| MspError::Crypto(e.to_string()))
    }

    pub fn public_key(&self) -> &PublicKey {
        self.keypair.public_key()
    }
}
