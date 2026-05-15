use thiserror::Error;

#[derive(Debug, Error)]
pub enum MspError {
    #[error("certificate verification failed: {0}")]
    CertVerificationFailed(String),
    #[error("certificate expired")]
    CertExpired,
    #[error("certificate not yet valid")]
    CertNotYetValid,
    #[error("certificate revoked (serial: {0})")]
    CertRevoked(String),
    #[error("token expired")]
    TokenExpired,
    #[error("token signature invalid")]
    TokenInvalidSignature,
    #[error("unknown certificate fingerprint: {0}")]
    UnknownCertFingerprint(String),
    #[error("no CA configured — running in dev mode")]
    NoCaConfigured,
    #[error("CA not found at {0}")]
    CaNotFound(String),
    #[error("user not found: {0}")]
    UnknownUser(String),
    #[error("user already exists: {0}")]
    UserAlreadyExists(String),
    #[error("invalid PEM format")]
    InvalidPem,
    #[error("wrong passphrase or corrupted key file")]
    WrongPassphrase,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("encryption error: {0}")]
    Encryption(String),
    #[error("crypto error: {0}")]
    Crypto(String),
}

impl From<bincode::Error> for MspError {
    fn from(e: bincode::Error) -> Self {
        MspError::Serialization(e.to_string())
    }
}
