use crate::ca::save_encrypted_keypair;
use crate::certificate::PQCertificate;
use crate::error::MspError;
use crate::identity::Identity;
use qorvum_crypto::signing::PQKeypair;
use std::path::{Path, PathBuf};

/// Persists enrolled user credentials (cert + password-encrypted keypair) under
/// `{ca_dir}/users/`.  The server holds keypairs on behalf of users so they can
/// authenticate with just a username + password over the REST API.
pub struct UserStore {
    dir: PathBuf,
}

impl UserStore {
    pub fn new(ca_dir: &Path) -> Self {
        let dir = ca_dir.join("users");
        std::fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    /// Persist an issued cert + keypair, encrypted with the user's password.
    pub fn enroll(
        &self,
        cert: &PQCertificate,
        keypair: &PQKeypair,
        password: &str,
    ) -> Result<(), MspError> {
        let name = &cert.subject.common_name;
        if self.exists(name) {
            return Err(MspError::UserAlreadyExists(name.clone()));
        }
        std::fs::write(self.dir.join(format!("{}.cert", name)), cert.to_pem_like())?;
        save_encrypted_keypair(keypair, password, &self.dir.join(format!("{}.key", name)))?;
        Ok(())
    }

    /// Load an Identity by decrypting the stored keypair with the user's password.
    /// Returns `WrongPassphrase` if the password is incorrect.
    pub fn load_identity(&self, username: &str, password: &str) -> Result<Identity, MspError> {
        let cert_path = self.dir.join(format!("{}.cert", username));
        let key_path = self.dir.join(format!("{}.key", username));
        if !cert_path.exists() {
            return Err(MspError::UnknownUser(username.to_string()));
        }
        Identity::load(&cert_path, &key_path, password)
    }

    /// Load the certificate for a user (no password needed — cert is public).
    pub fn get_cert(&self, username: &str) -> Result<PQCertificate, MspError> {
        let path = self.dir.join(format!("{}.cert", username));
        if !path.exists() {
            return Err(MspError::UnknownUser(username.to_string()));
        }
        PQCertificate::from_pem_like(&std::fs::read_to_string(&path)?)
    }

    pub fn exists(&self, username: &str) -> bool {
        self.dir.join(format!("{}.cert", username)).exists()
    }

    pub fn list_usernames(&self) -> Vec<String> {
        std::fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.strip_suffix(".cert").map(|s| s.to_string())
            })
            .collect()
    }
}
