//! Service-account API keys.

use crate::dto::{api_key_dto, ApiKeyDto, CreateKeyReq, CreatedKeyDto, ErrorBody};
use crate::error::{WebError, WebResult};
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use core_domain::entities::Scope;
use core_domain::ids::ApiKeyId;
use uuid::Uuid;

/// List the org's service-account API keys (the secret token is never returned).
#[utoipa::path(
    get, path = "/api/keys", tag = "keys",
    responses(
        (status = 200, description = "API keys", body = [ApiKeyDto]),
        (status = 403, description = "Missing keys.view", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> WebResult<Json<Vec<ApiKeyDto>>> {
    let keys = state.keys.list(&ctx).await?;
    Ok(Json(keys.iter().map(api_key_dto).collect()))
}

/// Create an API key. The full token is returned **once** in the response and
/// never again; only its prefix + hash are stored.
#[utoipa::path(
    post, path = "/api/keys", tag = "keys",
    request_body = CreateKeyReq,
    responses(
        (status = 200, description = "Created key + one-time token", body = CreatedKeyDto),
        (status = 400, description = "Unknown scope", body = ErrorBody),
        (status = 403, description = "Missing keys.create", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(req): Json<CreateKeyReq>,
) -> WebResult<Json<CreatedKeyDto>> {
    let scopes: Vec<Scope> = req
        .scopes
        .iter()
        .map(|s| {
            Scope::from_str(s).ok_or_else(|| WebError::bad_request(format!("unknown scope: {s}")))
        })
        .collect::<Result<_, _>>()?;
    let created = state.keys.create(&ctx, &req.name, scopes).await?;
    Ok(Json(CreatedKeyDto {
        key: api_key_dto(&created.key),
        token: created.token,
    }))
}

/// Revoke an API key by id. Idempotent.
#[utoipa::path(
    post, path = "/api/keys/{id}/revoke", tag = "keys",
    params(("id" = Uuid, Path, description = "API key id")),
    responses(
        (status = 204, description = "Key revoked"),
        (status = 403, description = "Missing keys.revoke", body = ErrorBody),
        (status = 404, description = "Key not found", body = ErrorBody),
    ),
    security(("session" = []))
)]
pub async fn revoke(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> WebResult<StatusCode> {
    state.keys.revoke(&ctx, ApiKeyId::from_uuid(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
