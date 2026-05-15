//! Qorvum REST Gateway
//! Exposes HTTP endpoints for contract invocation and ledger queries.

pub mod auth_handlers;
pub mod commit;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod sse;
pub mod state;
pub mod ws;

pub use routes::build_router;
pub use state::AppState;
pub use sse::EventBroadcaster;
