//! User identity endpoints.

use crate::dto::{
    user_dto, ChangeRoleReq, CreateUserReq, ErrorBody, InviteReq, SetStatusReq, UserDto,
};
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
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    /// Filter by role key (e.g. `admin`); `all` or empty means any.
    pub role: Option<String>,
    /// Filter by status: `active` · `invited` · `deactivated`; `all` means any.
    pub status: Option<String>,
    /// Case-insensitive substring over name + email.
    pub q: Option<String>,
}

/// List users in the org, filtered by role, status, and a free-text query.
#[utoipa::path(
    get, path = "/api/users", tag = "users",
    params(ListQuery),
    responses(
        (status = 200, description = "Matching users", body = [UserDto]),
        (status = 401, description = "Not signed in", body = ErrorBody),
        (status = 403, description = "Missing users.view", body = ErrorBody),
    ),
    security(("session" = []))
)]
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

/// Invite a user by email. Creates an `Invited` user and emails an accept-invite
/// link. Default role is Member.
#[utoipa::path(
    post, path = "/api/users/invite", tag = "users",
    request_body = InviteReq,
    responses(
        (status = 200, description = "Invited user", body = UserDto),
        (status = 403, description = "Missing users.invite or privilege escalation", body = ErrorBody),
        (status = 409, description = "Email already exists", body = ErrorBody),
        (status = 422, description = "Invalid email", body = ErrorBody),
    ),
    security(("session" = []))
)]
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
#[utoipa::path(
    post, path = "/api/users", tag = "users",
    request_body = CreateUserReq,
    responses(
        (status = 200, description = "Created user", body = UserDto),
        (status = 403, description = "Missing users.invite or privilege escalation", body = ErrorBody),
        (status = 409, description = "Email already exists", body = ErrorBody),
        (status = 422, description = "Invalid email or weak password", body = ErrorBody),
    ),
    security(("session" = []))
)]
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

/// Change a user's role. Revokes the target's sessions. Enforces
/// no-privilege-escalation and last-admin protection.
#[utoipa::path(
    patch, path = "/api/users/{id}/role", tag = "users",
    params(("id" = Uuid, Path, description = "User id")),
    request_body = ChangeRoleReq,
    responses(
        (status = 200, description = "Updated user", body = UserDto),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "User not found", body = ErrorBody),
        (status = 409, description = "Privilege escalation or last-admin protected", body = ErrorBody),
    ),
    security(("session" = []))
)]
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

/// Activate or deactivate a user. Deactivating revokes their sessions and is
/// blocked for the last remaining admin/owner.
#[utoipa::path(
    patch, path = "/api/users/{id}/status", tag = "users",
    params(("id" = Uuid, Path, description = "User id")),
    request_body = SetStatusReq,
    responses(
        (status = 200, description = "Updated user", body = UserDto),
        (status = 400, description = "Invalid status value", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 409, description = "Last-admin protected", body = ErrorBody),
    ),
    security(("session" = []))
)]
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
