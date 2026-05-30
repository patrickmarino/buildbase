//! Service-account API keys.

use crate::dto::{api_key_dto, CreateKeyReq, CreatedKeyDto};
use crate::error::{WebError, WebResult};
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use core_domain::entities::Scope;
use core_domain::ids::ApiKeyId;
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<Vec<crate::dto::ApiKeyDto>>> {
    let keys = state.keys.list(&ctx).await?;
    Ok(Json(keys.iter().map(api_key_dto).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<CreateKeyReq>,
) -> WebResult<Json<CreatedKeyDto>> {
    let scopes: Vec<Scope> = req
        .scopes
        .iter()
        .map(|s| Scope::from_str(s).ok_or_else(|| WebError::bad_request(format!("unknown scope: {s}"))))
        .collect::<Result<_, _>>()?;
    let created = state.keys.create(&ctx, &req.name, scopes).await?;
    Ok(Json(CreatedKeyDto {
        key: api_key_dto(&created.key),
        token: created.token,
    }))
}

pub async fn revoke(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> WebResult<StatusCode> {
    state.keys.revoke(&ctx, ApiKeyId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
