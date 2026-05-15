//! Unified API error type that converts to HTTP responses

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
    ChainError(chain_sdk::ChainError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(m)     => (StatusCode::NOT_FOUND,            "NOT_FOUND",     m),
            ApiError::BadRequest(m)   => (StatusCode::BAD_REQUEST,          "BAD_REQUEST",   m),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED,         "UNAUTHORIZED",  m),
            ApiError::Forbidden(m)    => (StatusCode::FORBIDDEN,            "FORBIDDEN",     m),
            ApiError::Internal(m)     => (StatusCode::INTERNAL_SERVER_ERROR,"INTERNAL_ERROR",m),
            ApiError::ChainError(e)   => {
                use chain_sdk::ChainError::*;
                let (s, c, m) = match e {
                    NotFound(m)         => (StatusCode::NOT_FOUND,            "NOT_FOUND",     m),
                    AlreadyExists(m)    => (StatusCode::CONFLICT,             "ALREADY_EXISTS",m),
                    ValidationFailed(m) => (StatusCode::BAD_REQUEST,          "VALIDATION",    m),
                    Unauthorized(m)     => (StatusCode::UNAUTHORIZED,         "UNAUTHORIZED",  m),
                    InternalError(m)    => (StatusCode::INTERNAL_SERVER_ERROR,"INTERNAL",      m),
                };
                (s, c, m)
            }
        };

        let body = Json(json!({
            "success": false,
            "error": { "code": code, "message": message }
        }));
        (status, body).into_response()
    }
}
