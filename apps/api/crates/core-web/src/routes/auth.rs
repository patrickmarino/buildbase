//! Authentication endpoints: login (sets the session cookie), logout, and
//! `me` (the SPA bootstrap payload).

use crate::cookies::{clear_cookie, session_cookie};
use crate::dto::{me_dto, AcceptInviteReq, LoginReq, MeDto};
use crate::error::{WebError, WebResult};
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::CookieJar;
use core_domain::ids::SessionId;

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginReq>,
) -> WebResult<(CookieJar, Json<MeDto>)> {
    let (_user, session) = state
        .auth
        .login(state.default_org, &req.email, &req.password)
        .await?;
    // Resolve the full context (role + matrix) for the bootstrap payload.
    let ctx = state
        .auth
        .resolve_actor(&session.id, None)
        .await
        .map_err(|_| WebError::unauthorized())?;
    let cookie = session_cookie(&state.cfg, session.id.0);
    Ok((jar.add(cookie), Json(me_dto(&ctx))))
}

/// Exchange an invitation token for a password + active account, and sign in.
pub async fn accept_invite(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<AcceptInviteReq>,
) -> WebResult<(CookieJar, Json<MeDto>)> {
    let (_user, session) = state.auth.accept_invite(&req.token, &req.password).await?;
    let ctx = state
        .auth
        .resolve_actor(&session.id, None)
        .await
        .map_err(|_| WebError::unauthorized())?;
    let cookie = session_cookie(&state.cfg, session.id.0);
    Ok((jar.add(cookie), Json(me_dto(&ctx))))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, StatusCode) {
    if let Some(c) = jar.get(&state.cfg.cookie_name) {
        let _ = state
            .auth
            .logout(&SessionId::new(c.value().to_string()))
            .await;
    }
    (jar.add(clear_cookie(&state.cfg)), StatusCode::NO_CONTENT)
}

pub async fn me(CurrentUser(ctx): CurrentUser) -> Json<MeDto> {
    Json(me_dto(&ctx))
}
