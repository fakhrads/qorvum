//! Router construction

use crate::{auth_handlers, handlers, middleware, state::AppState, ws};
use axum::{
    http::{HeaderValue, Method},
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn build_router(state: Arc<AppState>) -> Router {
    // ── CORS ──────────────────────────────────────────────────────────────────
    // SSE butuh CORS yang benar — browser kirim credentialed request
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        // Expose headers yang dibutuhkan SSE
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::CACHE_CONTROL,
        ]);

    // ── Public routes — tidak butuh token ────────────────────────────────────
    let public = Router::new()
        .route("/health",          get(handlers::health))
        .route("/stats",           get(handlers::get_stats))
        .route("/auth/login",      post(handlers::auth_login))
        .route("/metrics",         get(handlers::get_metrics))
        // Bootstrap hanya aktif kalau belum ada user — auto-reject setelahnya
        .route("/auth/bootstrap",  post(auth_handlers::auth_bootstrap))
        // WebSocket stream — real-time explorer, token optional via ?token=
        .route("/ws",              get(ws::ws_handler));

    // ── Protected routes — butuh Bearer token ────────────────────────────────
    let protected = Router::new()
        // Auth
        .route("/auth/refresh",                           post(auth_handlers::auth_refresh))
        // Contracts
        .route("/invoke/:contract/:function",             post(handlers::invoke))
        .route("/query/:contract/:function",              get(handlers::query))
        // Direct ledger
        .route("/records/:collection",                    get(handlers::list_records))
        .route("/records/:collection/:partition/:id",     get(handlers::get_record))
        .route("/history/:collection/:id",                get(handlers::get_history))
        // Block explorer — list + detail
        .route("/contracts",                              get(handlers::list_contracts))
        .route("/contracts/deploy",                       post(handlers::deploy_contract))
        .route("/blocks",                                 get(handlers::list_blocks))
        .route("/blocks/:number",                         get(handlers::get_block))
        // Admin
        .route("/admin/users",                            get(handlers::admin_list_users))
        .route("/admin/users/enroll",                     post(handlers::admin_enroll_user))
        .route("/admin/users/:username/revoke",           post(handlers::admin_revoke_user))
        .route("/admin/certs",                            get(handlers::admin_list_certs))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::identity_middleware,
        ));

    Router::new()
        .nest("/api/v1", public)
        .nest("/api/v1", protected)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}