//! # core-web
//!
//! The HTTP layer: Axum router, session-cookie auth, DTOs, and error→HTTP
//! mapping. It depends on `core-app` (use-cases) and `core-infra` (to wire the
//! concrete repositories into [`state::AppState`]).

pub mod config;
pub mod cookies;
pub mod dto;
pub mod error;
pub mod extractors;
pub mod openapi;
pub mod routes;
pub mod state;

pub use config::WebConfig;
pub use routes::build_router;
pub use state::AppState;
