//! User identity endpoints.

use crate::dto::{user_dto, ChangeRoleReq, CreateUserReq, InviteReq, SetStatusReq, UserDto};
use crate::error::{WebError, WebResult};
use crate::extractors::CurrentUser;
use crate::routes::roles_map;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use core_domain::entities::UserStatus;
use core_domain::ids::{RoleId, UserId};
use core_domain::ports::UserFilter;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub role: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<ListQuery>,
) -> WebResult<Json<Vec<UserDto>>> {
    let roles = roles_map(&state, &ctx).await?;
    let role_id = q
        .role
        .as_deref()
        .filter(|s| *s != "all" && !s.is_empty())
        .and_then(|s| roles.values().find(|r| r.key == s).map(|r| r.id));
    let filter = UserFilter {
        role_id,
        status: q
            .status
            .as_deref()
            .filter(|s| *s != "all")
            .and_then(UserStatus::from_str),
        query: q.q.filter(|s| !s.is_empty()),
    };
    let users = state.users.list(&ctx, filter).await?;
    Ok(Json(users.iter().map(|u| user_dto(u, &roles)).collect()))
}

pub async fn invite(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<InviteReq>,
) -> WebResult<Json<UserDto>> {
    let user = state
        .users
        .invite(&ctx, &req.email, &req.role, req.scope)
        .await?;
    let roles = roles_map(&state, &ctx).await?;
    Ok(Json(user_dto(&user, &roles)))
}

/// Manually create an active user with their basic info + an initial password.
pub async fn create(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<CreateUserReq>,
) -> WebResult<Json<UserDto>> {
    let user = state
        .users
        .create_user(
            &ctx,
            &req.name,
            &req.email,
            &req.role,
            req.scope,
            &req.password,
        )
        .await?;
    let roles = roles_map(&state, &ctx).await?;
    Ok(Json(user_dto(&user, &roles)))
}

pub async fn change_role(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeRoleReq>,
) -> WebResult<Json<UserDto>> {
    let user = state
        .users
        .change_role(&ctx, UserId::from_uuid(id), RoleId::from_uuid(req.role_id))
        .await?;
    let roles = roles_map(&state, &ctx).await?;
    Ok(Json(user_dto(&user, &roles)))
}

pub async fn set_status(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(req): Json<SetStatusReq>,
) -> WebResult<Json<UserDto>> {
    let status =
        UserStatus::from_str(&req.status).ok_or_else(|| WebError::bad_request("invalid status"))?;
    let user = state
        .users
        .set_status(&ctx, UserId::from_uuid(id), status)
        .await?;
    let roles = roles_map(&state, &ctx).await?;
    Ok(Json(user_dto(&user, &roles)))
}
