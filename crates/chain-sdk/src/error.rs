use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ChainError {
    #[error("AlreadyExists: {0}")]
    AlreadyExists(String),
    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("ValidationFailed: {0}")]
    ValidationFailed(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("InternalError: {0}")]
    InternalError(String),
}
