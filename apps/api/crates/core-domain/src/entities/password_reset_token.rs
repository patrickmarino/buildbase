//! Single-use, expiring password-reset tokens. The plaintext token is emailed to
//! the user; only its hash is stored, so the row can't be replayed if leaked.

use crate::ids::{PasswordResetTokenId, UserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: PasswordResetTokenId,
    pub user_id: UserId,
    /// SHA-256 hex of the plaintext token. The plaintext is never stored.
    #[serde(skip)]
    pub token_hash: String,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

impl PasswordResetToken {
    #[must_use]
    pub fn is_expired(&self, now: OffsetDateTime) -> bool {
        now >= self.expires_at
    }
}
