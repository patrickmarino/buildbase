//! Service-account API keys. The full token is shown exactly once on creation;
//! only a hash and a display prefix are stored.

use crate::ids::{ApiKeyId, OrgId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyStatus {
    Active,
    Revoked,
}

impl ApiKeyStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ApiKeyStatus::Active => "active",
            ApiKeyStatus::Revoked => "revoked",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "active" => ApiKeyStatus::Active,
            "revoked" => ApiKeyStatus::Revoked,
            _ => return None,
        })
    }
}

/// The fixed set of scopes a key can be granted (`SCOPE_OPTIONS` in the prototype).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[serde(rename = "users.view")]
    UsersView,
    #[serde(rename = "users.edit")]
    UsersEdit,
    #[serde(rename = "audit.view")]
    AuditView,
    #[serde(rename = "audit.export")]
    AuditExport,
    #[serde(rename = "roles.matrix")]
    RolesMatrix,
    #[serde(rename = "org.branding")]
    OrgBranding,
}

impl Scope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Scope::UsersView => "users.view",
            Scope::UsersEdit => "users.edit",
            Scope::AuditView => "audit.view",
            Scope::AuditExport => "audit.export",
            Scope::RolesMatrix => "roles.matrix",
            Scope::OrgBranding => "org.branding",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "users.view" => Scope::UsersView,
            "users.edit" => Scope::UsersEdit,
            "audit.view" => Scope::AuditView,
            "audit.export" => Scope::AuditExport,
            "roles.matrix" => Scope::RolesMatrix,
            "org.branding" => Scope::OrgBranding,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn all() -> [Scope; 6] {
        [
            Scope::UsersView,
            Scope::UsersEdit,
            Scope::AuditView,
            Scope::AuditExport,
            Scope::RolesMatrix,
            Scope::OrgBranding,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub org_id: OrgId,
    pub name: String,
    /// Display-only leading chars of the token, e.g. `ms_live_a91f`.
    pub prefix: String,
    /// SHA-256 hex of the full token. The plaintext token is never stored.
    #[serde(skip)]
    pub token_hash: String,
    pub scopes: Vec<Scope>,
    pub status: ApiKeyStatus,
    pub created_at: OffsetDateTime,
    pub last_used_at: Option<OffsetDateTime>,
}
