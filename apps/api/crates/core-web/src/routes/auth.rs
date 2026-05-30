//! Authentication endpoints: login (sets the session cookie), logout, and
//! `me` (the SPA bootstrap payload).

use crate::cookies::{clear_cookie, session_cookie};
use crate::dto::{
    me_dto, AcceptInviteReq, ErrorBody, ForgotPasswordReq, LoginReq, MeDto, ResetPasswordReq,
};
use crate::error::{WebError, WebResult};
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::CookieJar;
use core_domain::ids::SessionId;

/// Sign in with email + password. On success sets the `core_sid` session cookie
/// and returns the bootstrap payload (actor + resolved permissions).
#[utoipa::path(
    post, path = "/api/auth/login", tag = "auth",
    request_body = LoginReq,
    responses(
        (status = 200, description = "Signed in", body = MeDto),
        (status = 401, description = "Invalid credentials", body = ErrorBody),
    )
)]
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
#[utoipa::path(
    post, path = "/api/auth/accept-invite", tag = "auth",
    request_body = AcceptInviteReq,
    responses(
        (status = 200, description = "Account activated and signed in", body = MeDto),
        (status = 401, description = "Invalid or expired token", body = ErrorBody),
        (status = 422, description = "Password fails the org policy", body = ErrorBody),
    )
)]
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
#[utoipa::path(
    post, path = "/api/auth/forgot-password", tag = "auth",
    request_body = ForgotPasswordReq,
    responses((status = 204, description = "Reset email sent if the account exists"))
)]
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
#[utoipa::path(
    post, path = "/api/auth/reset-password", tag = "auth",
    request_body = ResetPasswordReq,
    responses(
        (status = 200, description = "Password reset and signed in", body = MeDto),
        (status = 401, description = "Invalid or expired token", body = ErrorBody),
        (status = 422, description = "Password fails the org policy", body = ErrorBody),
    )
)]
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

/// Sign out: revoke the current session and clear the cookie. Idempotent.
#[utoipa::path(
    post, path = "/api/auth/logout", tag = "auth",
    responses((status = 204, description = "Signed out")),
    security(("session" = []))
)]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, StatusCode) {
    if let Some(c) = jar.get(&state.cfg.cookie_name) {
        let _ = state
            .auth
            .logout(&SessionId::new(c.value().to_string()))
            .await;
    }
    (jar.add(clear_cookie(&state.cfg)), StatusCode::NO_CONTENT)
}

/// The signed-in actor plus the permission keys they may exercise — the SPA
/// bootstrap payload.
#[utoipa::path(
    get, path = "/api/auth/me", tag = "auth",
    responses(
        (status = 200, description = "Current actor", body = MeDto),
        (status = 401, description = "Not signed in", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn me(CurrentUser(ctx): CurrentUser) -> Json<MeDto> {
    Json(me_dto(&ctx))
}
