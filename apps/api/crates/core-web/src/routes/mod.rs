//! HTTP routes. Auth uses a session cookie; the `CurrentUser` extractor guards
//! every protected handler. All endpoints are mounted under `/api`.

mod audit;
mod auth;
mod keys;
mod org;
mod roles;
mod users;

use crate::error::WebError;
use crate::state::AppState;
use axum::routing::{get, patch, post};
use axum::Router;
use core_app::ActorContext;
use core_domain::entities::Role;
use core_domain::ids::RoleId;
use std::collections::HashMap;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            state.cfg.web_origin.parse().expect("valid WEB_ORIGIN"),
        ))
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let api = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/users", get(users::list))
        .route("/users/invite", post(users::invite))
        .route("/users/{id}/role", patch(users::change_role))
        .route("/users/{id}/status", patch(users::set_status))
        .route("/roles", get(roles::list).post(roles::create))
        .route("/permissions/matrix", get(roles::matrix))
        .route("/permissions/matrix/cell", patch(roles::cycle_cell))
        .route("/org", get(org::get_org).patch(org::update).delete(org::delete))
        .route("/org/transfer-ownership", post(org::transfer))
        .route("/org/transfer-ownership/accept", post(org::accept))
        .route("/audit", get(audit::list))
        .route("/audit/export", get(audit::export))
        .route("/keys", get(keys::list).post(keys::create))
        .route("/keys/{id}/revoke", post(keys::revoke))
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "ok"
}

/// Fetch the org's roles as an id→role map, for DTO mapping. Reuses the guarded
/// `RoleService::list` (anyone who can reach these endpoints can view roles).
pub(crate) async fn roles_map(
    state: &AppState,
    ctx: &ActorContext,
) -> Result<HashMap<RoleId, Role>, WebError> {
    let roles = state.roles.list(ctx).await?;
    Ok(roles.into_iter().map(|r| (r.id, r)).collect())
}
