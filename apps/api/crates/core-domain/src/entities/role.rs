//! Roles. Default (builtin) roles plus org-defined custom roles. Roles carry a
//! numeric `rank` that drives both downward inheritance (a higher role includes
//! everything below it) and the no-privilege-escalation guard.

use crate::ids::{OrgId, RoleId};
use serde::{Deserialize, Serialize};

/// The seven default roles from the PRD. The first five are *matrix columns*;
/// `Guest` and `Service` appear in the roster but are not permission columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinRole {
    Owner,
    Admin,
    Manager,
    Member,
    Viewer,
    Guest,
    Service,
}

impl BuiltinRole {
    /// The stable key used in the DB and API (e.g. `"owner"`).
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            BuiltinRole::Owner => "owner",
            BuiltinRole::Admin => "admin",
            BuiltinRole::Manager => "manager",
            BuiltinRole::Member => "member",
            BuiltinRole::Viewer => "viewer",
            BuiltinRole::Guest => "guest",
            BuiltinRole::Service => "service",
        }
    }

    /// Rank: higher = more powerful. Drives inheritance and escalation checks.
    /// Spacing leaves room for custom roles to slot between tiers if needed.
    #[must_use]
    pub const fn rank(self) -> i16 {
        match self {
            BuiltinRole::Owner => 60,
            BuiltinRole::Admin => 50,
            BuiltinRole::Manager => 40,
            BuiltinRole::Member => 30,
            BuiltinRole::Viewer => 20,
            BuiltinRole::Guest => 10,
            BuiltinRole::Service => 10,
        }
    }

    /// Display name shown in the UI.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            BuiltinRole::Owner => "Owner",
            BuiltinRole::Admin => "Admin",
            BuiltinRole::Manager => "Manager",
            BuiltinRole::Member => "Member",
            BuiltinRole::Viewer => "Viewer",
            BuiltinRole::Guest => "Guest",
            BuiltinRole::Service => "Service account",
        }
    }

    /// All seven defaults, most → least powerful (columns first, then guest/service).
    #[must_use]
    pub const fn ordered() -> [BuiltinRole; 7] {
        [
            BuiltinRole::Owner,
            BuiltinRole::Admin,
            BuiltinRole::Manager,
            BuiltinRole::Member,
            BuiltinRole::Viewer,
            BuiltinRole::Guest,
            BuiltinRole::Service,
        ]
    }

    #[must_use]
    pub const fn is_column(self) -> bool {
        matches!(
            self,
            BuiltinRole::Owner
                | BuiltinRole::Admin
                | BuiltinRole::Manager
                | BuiltinRole::Member
                | BuiltinRole::Viewer
        )
    }

    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "owner" => BuiltinRole::Owner,
            "admin" => BuiltinRole::Admin,
            "manager" => BuiltinRole::Manager,
            "member" => BuiltinRole::Member,
            "viewer" => BuiltinRole::Viewer,
            "guest" => BuiltinRole::Guest,
            "service" => BuiltinRole::Service,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub org_id: OrgId,
    /// Stable slug, e.g. `"owner"` or a custom `"stager"`.
    pub key: String,
    pub name: String,
    /// `Some(..)` for the seven defaults; `None` for custom roles.
    pub builtin: Option<BuiltinRole>,
    /// Whether this role is a column in the permission matrix.
    pub is_column: bool,
    /// Custom roles inherit (copy + tune) from a base role.
    pub base_role_id: Option<RoleId>,
    /// Higher = more powerful (see [`BuiltinRole::rank`]).
    pub rank: i16,
}

impl Role {
    #[must_use]
    pub fn is_owner(&self) -> bool {
        self.builtin == Some(BuiltinRole::Owner)
    }

    #[must_use]
    pub fn is_custom(&self) -> bool {
        self.builtin.is_none()
    }
}
