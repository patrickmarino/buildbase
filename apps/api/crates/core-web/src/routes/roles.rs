//! Roles and the permission matrix.

use crate::dto::{
    cell_result_dto, matrix_dto, role_dto, CellResultDto, CreateRoleReq, CycleCellReq, ErrorBody,
    MatrixDto, RoleDto,
};
use crate::error::WebResult;
use crate::extractors::CurrentUser;
use crate::routes::roles_map;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use core_domain::ids::RoleId;

/// All roles in the org (column roles + guest/service), most→least powerful.
#[utoipa::path(
    get, path = "/api/roles", tag = "roles",
    responses(
        (status = 200, description = "Roles", body = [RoleDto]),
        (status = 401, description = "Not signed in", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<Vec<RoleDto>>> {
    let roles = state.roles.list(&ctx).await?;
    let map = roles.iter().map(|r| (r.id, r.clone())).collect();
    Ok(Json(roles.iter().map(|r| role_dto(r, &map)).collect()))
}

/// Create a custom role that inherits a base role's cells (a new matrix column).
#[utoipa::path(
    post, path = "/api/roles", tag = "roles",
    request_body = CreateRoleReq,
    responses(
        (status = 200, description = "Created role", body = RoleDto),
        (status = 403, description = "Missing roles.create", body = ErrorBody),
        (status = 409, description = "Duplicate role name", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<CreateRoleReq>,
) -> WebResult<Json<RoleDto>> {
    let role = state
        .roles
        .create_custom(&ctx, &req.name, RoleId::from_uuid(req.base_role_id))
        .await?;
    let map = roles_map(&state, &ctx).await?;
    Ok(Json(role_dto(&role, &map)))
}

/// The full permission matrix: groups/actions, column roles, and cell states.
#[utoipa::path(
    get, path = "/api/permissions/matrix", tag = "roles",
    responses(
        (status = 200, description = "Permission matrix", body = MatrixDto),
        (status = 401, description = "Not signed in", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn matrix(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<MatrixDto>> {
    let matrix = state.permissions.get_matrix(&ctx).await?;
    let roles = roles_map(&state, &ctx).await?;
    Ok(Json(matrix_dto(&matrix, &roles)))
}

/// Cycle one matrix cell's state (allow → scope → deny). Locked cells are
/// rejected with 409.
#[utoipa::path(
    patch, path = "/api/permissions/matrix/cell", tag = "roles",
    request_body = CycleCellReq,
    responses(
        (status = 200, description = "New cell state", body = CellResultDto),
        (status = 403, description = "Missing permission.edit", body = ErrorBody),
        (status = 409, description = "Cell is locked", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn cycle_cell(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<CycleCellReq>,
) -> WebResult<Json<CellResultDto>> {
    let cell = state
        .permissions
        .cycle_cell(&ctx, &req.action_key, RoleId::from_uuid(req.role_id))
        .await?;
    Ok(Json(cell_result_dto(&cell)))
}
