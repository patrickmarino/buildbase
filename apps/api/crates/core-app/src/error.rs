//! Application errors. Wraps [`DomainError`] plus an `Unauthorized` case that has
//! no domain meaning (it's about the *session*, not a business rule). The web
//! layer maps these to HTTP status codes.

use core_domain::error::DomainError;
use core_domain::ports::RepoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    /// No valid session / not signed in.
    #[error("unauthorized")]
    Unauthorized,

    /// Any domain-level error (forbidden, not found, conflict, rule violation…).
    #[error(transparent)]
    Domain(#[from] DomainError),
}

impl From<RepoError> for AppError {
    fn from(e: RepoError) -> Self {
        AppError::Domain(DomainError::from(e))
    }
}

impl AppError {
    /// The inner domain error, if any — convenient for the web layer's status mapping.
    #[must_use]
    pub fn as_domain(&self) -> Option<&DomainError> {
        match self {
            AppError::Domain(d) => Some(d),
            AppError::Unauthorized => None,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
