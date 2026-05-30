//! Roles and the permission matrix.

use crate::dto::{
    cell_result_dto, matrix_dto, role_dto, CellResultDto, CreateRoleReq, CycleCellReq, MatrixDto,
    RoleDto,
};
use crate::error::WebResult;
use crate::extractors::CurrentUser;
use crate::routes::roles_map;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use core_domain::ids::RoleId;

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<Vec<RoleDto>>> {
    let roles = state.roles.list(&ctx).await?;
    let map = roles.iter().map(|r| (r.id, r.clone())).collect();
    Ok(Json(roles.iter().map(|r| role_dto(r, &map)).collect()))
}

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

pub async fn matrix(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<MatrixDto>> {
    let matrix = state.permissions.get_matrix(&ctx).await?;
    let roles = roles_map(&state, &ctx).await?;
    Ok(Json(matrix_dto(&matrix, &roles)))
}

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
