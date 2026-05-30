//! Audit log: search/filter and CSV export.

use crate::dto::{audit_dto, AuditDto};
use crate::error::WebResult;
use crate::extractors::CurrentUser;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use core_domain::entities::PermissionCategory;
use core_domain::ports::AuditQuery;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub page: Option<i64>,
}

fn to_query(q: &SearchQuery) -> AuditQuery {
    let page = q.page.unwrap_or(0).max(0);
    let limit = 200;
    AuditQuery {
        category: q
            .category
            .as_deref()
            .filter(|c| *c != "all")
            .and_then(PermissionCategory::from_str),
        query: q.q.clone().filter(|s| !s.is_empty()),
        limit,
        offset: page * limit,
    }
}

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<SearchQuery>,
) -> WebResult<Json<Vec<AuditDto>>> {
    let events = state.audit.search(&ctx, to_query(&q)).await?;
    Ok(Json(events.iter().map(audit_dto).collect()))
}

pub async fn export(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<SearchQuery>,
) -> WebResult<impl IntoResponse> {
    let csv = state.audit.export_csv(&ctx, to_query(&q)).await?;
    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (header::CONTENT_DISPOSITION, "attachment; filename=\"audit-log.csv\""),
    ];
    Ok((headers, csv))
}
