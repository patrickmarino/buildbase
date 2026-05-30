//! Ports — the seams between the domain and the outside world.
//!
//! Repository traits (data access) plus the small capability traits the
//! use-cases need (`PasswordHasher`, `TokenGenerator`, `Clock`). Infrastructure
//! implements these; the application layer depends only on the traits, which is
//! what makes the use-cases unit-testable with in-memory fakes.

mod repos;

pub use repos::{
    ApiKeyRepo, AuditQuery, AuditRepo, OrgRepo, PermissionRepo, RoleRepo, SessionRepo, UserFilter,
    UserRepo,
};

use crate::entities::organization::PasswordPolicy;
use crate::error::DomainError;
use crate::ids::SessionId;
use thiserror::Error;
use time::OffsetDateTime;

/// A failure surfaced by a repository implementation.
#[derive(Debug, Error)]
pub enum RepoError {
    /// A unique constraint was violated (e.g. duplicate email or role key).
    #[error("unique violation: {0}")]
    UniqueViolation(String),
    /// Any other backend failure (connection, query, mapping…).
    #[error("repository failure: {0}")]
    Other(String),
}

impl From<RepoError> for DomainError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::UniqueViolation(m) => DomainError::Conflict(m),
            RepoError::Other(m) => DomainError::Repo(m),
        }
    }
}

pub type RepoResult<T> = Result<T, RepoError>;

/// A freshly minted API token: the full secret (shown once) and its public prefix.
#[derive(Debug, Clone)]
pub struct ApiToken {
    pub full: String,
    pub prefix: String,
}

/// Password hashing (Argon2 in infra). Synchronous: hashing is CPU-bound and
/// the caller decides whether to offload it.
pub trait PasswordHasher: Send + Sync {
    /// Hash a plaintext password, returning a PHC string.
    fn hash(&self, plaintext: &str) -> Result<String, DomainError>;
    /// Constant-time verify of a plaintext against a stored PHC hash.
    fn verify(&self, plaintext: &str, phc: &str) -> Result<bool, DomainError>;
}

/// CSPRNG-backed generation of opaque identifiers and tokens.
pub trait TokenGenerator: Send + Sync {
    /// A high-entropy session id (the cookie value).
    fn new_session_id(&self) -> SessionId;
    /// A new API token. `full` is returned to the caller once; infra stores only
    /// a hash of it plus the `prefix`.
    fn new_api_token(&self) -> ApiToken;
    /// Hash an API token for storage / lookup (SHA-256 hex).
    fn hash_api_token(&self, full: &str) -> String;
}

/// The clock. Injectable so tests can pin time.
pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}

/// A plain-text email to send.
#[derive(Debug, Clone)]
pub struct OutgoingEmail {
    pub to_address: String,
    pub to_name: String,
    pub subject: String,
    pub body: String,
}

/// An email-delivery failure (SMTP, formatting, …).
#[derive(Debug, Error)]
#[error("email send failed: {0}")]
pub struct EmailError(pub String);

/// Outbound email. Infra delivers via SMTP (Mailpit in dev); tests capture.
#[async_trait::async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, email: &OutgoingEmail) -> Result<(), EmailError>;
}

/// Convenience: a policy-aware password check used by the auth/use-case layer.
/// (The pure rule lives in [`crate::services::password_policy`].)
pub fn assert_password_ok(pw: &str, policy: &PasswordPolicy) -> Result<(), DomainError> {
    crate::services::password_policy::validate_password(pw, policy)
        .map_err(DomainError::PasswordPolicy)
}
