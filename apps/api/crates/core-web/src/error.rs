//! Web error type and its mapping to HTTP responses. Produces a stable JSON
//! body `{ "error": <code>, "message": <msg> }` so the SPA can branch on `code`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use core_app::AppError;
use core_domain::error::DomainError;
use serde_json::json;

pub struct WebError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl WebError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", "not signed in")
    }
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let body = Json(json!({ "error": self.code, "message": self.message }));
        (self.status, body).into_response()
    }
}

impl From<AppError> for WebError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Unauthorized => WebError::unauthorized(),
            AppError::Domain(d) => domain_to_web(d),
        }
    }
}

fn domain_to_web(d: DomainError) -> WebError {
    use DomainError::*;
    let (status, code) = match &d {
        Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
        NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
        Conflict(_) => (StatusCode::CONFLICT, "conflict"),
        PrivilegeEscalation(_) => (StatusCode::CONFLICT, "privilege_escalation"),
        LastAdminProtected(_) => (StatusCode::CONFLICT, "last_admin_protected"),
        CellLocked(_) => (StatusCode::CONFLICT, "cell_locked"),
        PasswordPolicy(_) => (StatusCode::UNPROCESSABLE_ENTITY, "password_policy"),
        Invalid(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid"),
        Repo(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
    };
    let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
        // Don't leak internal detail.
        "internal server error".to_string()
    } else {
        d.to_string()
    };
    WebError {
        status,
        code,
        message,
    }
}

pub type WebResult<T> = Result<T, WebError>;
