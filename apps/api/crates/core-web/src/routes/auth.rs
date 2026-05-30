//! Authentication endpoints: login (sets the session cookie), logout, and
//! `me` (the SPA bootstrap payload).

use crate::cookies::{clear_cookie, session_cookie};
use crate::dto::{me_dto, AcceptInviteReq, ForgotPasswordReq, LoginReq, MeDto, ResetPasswordReq};
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

/// Begin a self-serve password reset. Always returns 204 regardless of whether
/// the email matches a user, so the endpoint can't be used to enumerate accounts.
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordReq>,
) -> WebResult<StatusCode> {
    state
        .auth
        .request_password_reset(state.default_org, &req.email)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Complete a password reset with a valid token, then sign the user in.
pub async fn reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<ResetPasswordReq>,
) -> WebResult<(CookieJar, Json<MeDto>)> {
    let (_user, session) = state.auth.reset_password(&req.token, &req.password).await?;
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
