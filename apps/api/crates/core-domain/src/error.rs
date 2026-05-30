//! Domain-level errors — business rule violations and lookups. No HTTP, no SQL.

use serde::Serialize;
use thiserror::Error;

/// A single password-policy violation, returned as a list so the UI can show
/// every failing rule at once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum PolicyViolation {
    TooShort { min: u8, actual: usize },
    MissingNumber,
    MissingSymbol,
}

impl std::fmt::Display for PolicyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyViolation::TooShort { min, actual } => {
                write!(f, "must be at least {min} characters (got {actual})")
            }
            PolicyViolation::MissingNumber => write!(f, "must contain a number"),
            PolicyViolation::MissingSymbol => write!(f, "must contain a symbol"),
        }
    }
}

/// Errors produced by domain rules and (via the ports) repositories.
#[derive(Debug, Error)]
pub enum DomainError {
    /// The actor tried to assign or reach a privilege above their own level.
    #[error("privilege escalation: {0}")]
    PrivilegeEscalation(String),

    /// Removing/demoting this principal would leave the org without a required
    /// administrator (or sole owner).
    #[error("last-admin protection: {0}")]
    LastAdminProtected(String),

    /// A matrix cell is locked by rule and cannot be edited.
    #[error("permission cell is locked: {0}")]
    CellLocked(String),

    /// Password failed one or more policy rules.
    #[error("password policy not satisfied")]
    PasswordPolicy(Vec<PolicyViolation>),

    /// A required entity was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// The actor is not permitted to perform this action (authz denied).
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// A value or transition is invalid (validation / bad state).
    #[error("invalid: {0}")]
    Invalid(String),

    /// A uniqueness/state conflict (e.g. duplicate email).
    #[error("conflict: {0}")]
    Conflict(String),

    /// An underlying repository/infrastructure failure surfaced through a port.
    #[error("repository error: {0}")]
    Repo(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
