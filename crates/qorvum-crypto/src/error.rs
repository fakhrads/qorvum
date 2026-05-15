use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGenFailed(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Verification failed: invalid signature")]
    VerificationFailed,
    #[error("Invalid key bytes: {0}")]
    InvalidKey(String),
    #[error("KEM encapsulation failed: {0}")]
    KemFailed(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}
