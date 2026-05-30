//! Mapping sqlx errors onto the domain's [`RepoError`].

use core_domain::ports::RepoError;

/// Translate an `sqlx::Error` into a `RepoError`, detecting unique-constraint
/// violations (Postgres SQLSTATE 23505) so the app layer can surface a 409.
pub fn map_sqlx(e: sqlx::Error) -> RepoError {
    if let sqlx::Error::Database(db) = &e {
        if db.code().as_deref() == Some("23505") {
            return RepoError::UniqueViolation(db.message().to_string());
        }
    }
    RepoError::Other(e.to_string())
}

/// A value that could not be decoded into a domain enum/type.
pub fn decode_err(what: &str, value: &str) -> RepoError {
    RepoError::Other(format!("invalid {what} in database: {value:?}"))
}
