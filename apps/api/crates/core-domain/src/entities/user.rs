//! Users — human identities within an organization.

use crate::ids::{OrgId, RoleId, UserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Invited,
    Deactivated,
}

impl UserStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Invited => "invited",
            UserStatus::Deactivated => "deactivated",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "active" => UserStatus::Active,
            "invited" => UserStatus::Invited,
            "deactivated" => UserStatus::Deactivated,
            _ => return None,
        })
    }
}

use crate::entities::email::Email;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub org_id: OrgId,
    pub email: Email,
    pub name: String,
    /// The user's current primary role.
    pub role_id: RoleId,
    pub status: UserStatus,
    /// Free-text scope label (team/department), e.g. "Hampstead". Mirrors the
    /// prototype; deeper scoping is a future concern.
    pub scope: Option<String>,
    /// Argon2 PHC string. `None` for invited users who haven't set a password.
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: OffsetDateTime,
    pub last_active_at: Option<OffsetDateTime>,
}

impl User {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }
}
