//! Organization settings, ownership transfer, and deletion.

use crate::dto::{org_dto, OrgDto, TransferReq, UpdateOrgReq};
use crate::error::WebResult;
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use core_app::OrgSettingsPatch;
use core_domain::ids::UserId;

pub async fn get_org(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<OrgDto>> {
    Ok(Json(org_dto(&state.org.get(&ctx).await?)))
}

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

pub async fn accept(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<StatusCode> {
    state.org.accept_ownership(&ctx).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<StatusCode> {
    state.org.delete_org(&ctx).await?;
    Ok(StatusCode::NO_CONTENT)
}
