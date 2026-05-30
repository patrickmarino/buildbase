//! Organization settings, ownership transfer, and deletion.

use crate::dto::{org_dto, ErrorBody, OrgDto, TransferReq, UpdateOrgReq};
use crate::error::WebResult;
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use core_app::OrgSettingsPatch;
use core_domain::ids::UserId;

/// The organization profile, branding, MFA, password policy, and SSO settings.
#[utoipa::path(
    get, path = "/api/org", tag = "org",
    responses(
        (status = 200, description = "Organization", body = OrgDto),
        (status = 401, description = "Not signed in", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn get_org(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<OrgDto>> {
    Ok(Json(org_dto(&state.org.get(&ctx).await?)))
}

/// Update org settings (partial — only provided fields change).
#[utoipa::path(
    patch, path = "/api/org", tag = "org",
    request_body = UpdateOrgReq,
    responses(
        (status = 200, description = "Updated organization", body = OrgDto),
        (status = 403, description = "Missing org.edit", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<UpdateOrgReq>,
) -> WebResult<Json<OrgDto>> {
    let patch = OrgSettingsPatch {
        name: req.name,
        domain: req.domain,
        accent_color: req.accent_color,
        mfa: req.mfa.map(|m| m.into_domain()),
        password_policy: req.password_policy.map(|p| p.into_domain()),
        sso: req.sso.map(|s| s.into_domain()),
    };
    Ok(Json(org_dto(&state.org.update(&ctx, patch).await?)))
}

/// Begin an ownership transfer to another user (they must accept).
#[utoipa::path(
    post, path = "/api/org/transfer-ownership", tag = "org",
    request_body = TransferReq,
    responses(
        (status = 204, description = "Transfer initiated"),
        (status = 403, description = "Only the owner may transfer", body = ErrorBody),
        (status = 404, description = "Target user not found", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn transfer(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<TransferReq>,
) -> WebResult<StatusCode> {
    state
        .org
        .transfer_ownership(&ctx, UserId::from_uuid(req.target_user_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Accept a pending ownership transfer (called by the new owner).
#[utoipa::path(
    post, path = "/api/org/transfer-ownership/accept", tag = "org",
    responses(
        (status = 204, description = "Ownership accepted"),
        (status = 403, description = "No pending transfer for this user", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn accept(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<StatusCode> {
    state.org.accept_ownership(&ctx).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Permanently delete the organization and all dependent data. Owner only.
#[utoipa::path(
    delete, path = "/api/org", tag = "org",
    responses(
        (status = 204, description = "Organization deleted"),
        (status = 403, description = "Only the owner may delete", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<StatusCode> {
    state.org.delete_org(&ctx).await?;
    Ok(StatusCode::NO_CONTENT)
}
