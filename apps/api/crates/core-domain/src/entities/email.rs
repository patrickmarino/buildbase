//! A validated email value object. Constructed only through [`Email::parse`],
//! so an `Email` in the domain is always syntactically valid and normalized to
//! lowercase.

use crate::error::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Email(String);

impl Email {
    /// Parse and normalize an email. Mirrors the prototype's client-side regex
    /// (`^[^@\s]+@[^@\s]+\.[^@\s]+$`) so backend and frontend agree on validity.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if is_valid_email(trimmed) {
            Ok(Self(trimmed.to_ascii_lowercase()))
        } else {
            Err(DomainError::Invalid(format!("invalid email: {raw}")))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    /// The local part before `@` — used to derive a display name on invite,
    /// matching the prototype (`name.split('@')[0]`).
    #[must_use]
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or(&self.0)
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `something@something.tld` with no whitespace and exactly one `@`.
fn is_valid_email(s: &str) -> bool {
    let mut parts = s.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    if local.is_empty() || s.chars().any(char::is_whitespace) {
        return false;
    }
    // domain must contain a dot with non-empty labels on both sides
    match domain.rsplit_once('.') {
        Some((host, tld)) => !host.is_empty() && !tld.is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_lowercases_valid() {
        let e = Email::parse("  Elena@MadeSpace.CO ").unwrap();
        assert_eq!(e.as_str(), "elena@madespace.co");
        assert_eq!(e.local_part(), "elena");
    }

    #[test]
    fn rejects_invalid() {
        for bad in ["nope", "a@b", "@b.co", "a@.co", "a b@c.co", "a@b c.co", "a@@b.co"] {
            assert!(Email::parse(bad).is_err(), "should reject {bad:?}");
        }
    }
}
