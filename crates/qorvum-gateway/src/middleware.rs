use axum::{extract::Request, extract::State, middleware::Next, response::Response};
use std::sync::Arc;
use tracing::{info, warn};

use crate::state::AppState;

/// Caller identity injected by middleware into request extensions.
#[derive(Clone, Debug)]
pub struct CallerIdentity {
    pub id: String,
    pub roles: Vec<String>,
    pub org: String,
    pub verified: bool,
}

/// Extracts and verifies identity from `Authorization: Bearer <token>`.
/// Falls back to dev-mode (X-Identity/X-Roles headers) when no CA is configured.
pub async fn identity_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let caller = match state.verifier.as_ref() {
        Some(verifier_lock) => {
            // Production mode — require a verified token
            let bearer = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.trim().to_string());

            match bearer {
                None => {
                    return axum::response::Response::builder()
                        .status(401)
                        .header("Content-Type", "application/json")
                        .body(axum::body::Body::from(
                            r#"{"success":false,"error":{"code":"UNAUTHORIZED","message":"Missing Authorization header"}}"#,
                        ))
                        .unwrap();
                }
                Some(token_str) => {
                    let verifier = verifier_lock.read().await;
                    match verifier.verify_token(&token_str) {
                        Ok(verified) => {
                            info!(
                                subject = %verified.subject,
                                org = %verified.org,
                                "Authenticated via PQ token"
                            );
                            CallerIdentity {
                                id: format!("{}@{}", verified.subject, verified.org),
                                roles: verified.roles,
                                org: verified.org,
                                verified: true,
                            }
                        }
                        Err(e) => {
                            warn!("Token verification failed: {}", e);
                            return axum::response::Response::builder()
                                .status(401)
                                .header("Content-Type", "application/json")
                                .body(axum::body::Body::from(format!(
                                    r#"{{"success":false,"error":{{"code":"UNAUTHORIZED","message":"{}"}}}}"#,
                                    e
                                )))
                                .unwrap();
                        }
                    }
                }
            }
        }
        None => {
            // Dev mode — accept X-Identity / X-Roles headers without verification
            let identity = req
                .headers()
                .get("X-Identity")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("anonymous@default")
                .to_string();

            let roles_header = req
                .headers()
                .get("X-Roles")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            CallerIdentity {
                id: identity.clone(),
                roles: roles_header
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect(),
                org: identity
                    .split('@')
                    .nth(1)
                    .unwrap_or("default")
                    .to_string(),
                verified: false,
            }
        }
    };

    req.extensions_mut().insert(caller);
    next.run(req).await
}
