use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Record already exists: {0}")]
    AlreadyExists(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Block error: {0}")]
    BlockError(String),
    #[error("Delta computation error: {0}")]
    DeltaError(String),
    #[error("Delta hash mismatch — integrity check failed")]
    HashMismatch,
}
