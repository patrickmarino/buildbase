//! Request extractors. `CurrentUser` resolves the acting principal from the
//! session cookie — its presence on a handler *is* the auth guard.

use crate::error::WebError;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use core_app::ActorContext;
use core_domain::ids::SessionId;

/// A resolved, authenticated actor. Handlers that take this require a valid
/// session; a missing/invalid/expired cookie yields 401 before the handler runs.
pub struct CurrentUser(pub ActorContext);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, WebError> {
        let jar = CookieJar::from_headers(&parts.headers);
        let sid = jar
            .get(&state.cfg.cookie_name)
            .map(|c| c.value().to_string())
            .ok_or_else(WebError::unauthorized)?;
        let ip = client_ip(parts);
        let ctx = state
            .auth
            .resolve_actor(&SessionId::new(sid), ip)
            .await
            .map_err(|_| WebError::unauthorized())?;
        Ok(CurrentUser(ctx))
    }
}

/// Best-effort client IP from common proxy headers, for audit records.
pub fn client_ip(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            parts
                .headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
}
