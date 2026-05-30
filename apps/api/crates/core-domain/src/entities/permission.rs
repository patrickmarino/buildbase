//! The permission matrix: the single source of truth for "who can do what".
//!
//! Conceptually a grid — rows are *actions* (grouped into categories), columns
//! are *column roles*, and each cell holds a [`PermissionState`]. Roles inherit
//! downward by rank (a higher role includes everything a lower role can do),
//! which [`PermissionMatrix::effective_state`] resolves.

use crate::ids::RoleId;
use serde::{Deserialize, Serialize};

/// The three states a cell can hold. Ordered by permissiveness:
/// `Deny < Scope < Allow` — used when resolving inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Deny,
    Scope,
    Allow,
}

impl PermissionState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PermissionState::Allow => "allow",
            PermissionState::Scope => "scope",
            PermissionState::Deny => "deny",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "allow" => PermissionState::Allow,
            "scope" => PermissionState::Scope,
            "deny" => PermissionState::Deny,
            _ => return None,
        })
    }

    /// Human label matching the prototype (`STATE_LABEL`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PermissionState::Allow => "Allowed",
            PermissionState::Scope => "Scoped",
            PermissionState::Deny => "Denied",
        }
    }
}

/// The five matrix groups, mirroring `SEED_GROUPS` in the prototype's `data.jsx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionCategory {
    Users,
    Roles,
    Org,
    Audit,
    Keys,
}

impl PermissionCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PermissionCategory::Users => "users",
            PermissionCategory::Roles => "roles",
            PermissionCategory::Org => "org",
            PermissionCategory::Audit => "audit",
            PermissionCategory::Keys => "keys",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PermissionCategory::Users => "Users",
            PermissionCategory::Roles => "Roles & permissions",
            PermissionCategory::Org => "Organization settings",
            PermissionCategory::Audit => "Audit log",
            PermissionCategory::Keys => "Service accounts & API keys",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "users" => PermissionCategory::Users,
            "roles" => PermissionCategory::Roles,
            "org" => PermissionCategory::Org,
            "audit" => PermissionCategory::Audit,
            "keys" => PermissionCategory::Keys,
            _ => return None,
        })
    }
}

/// A single action a role may be permitted to perform, e.g. `users.invite`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub key: String,
    pub category: PermissionCategory,
    pub label: String,
}

/// One cell of the grid: the state of `action_key` for `role_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixCell {
    pub action_key: String,
    pub role_id: RoleId,
    pub state: PermissionState,
}

/// A category header plus its ordered actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub category: PermissionCategory,
    pub actions: Vec<Action>,
}

/// The full matrix for an org: groups (rows), column roles, and the cell states.
/// Also carries each column role's rank so inheritance can be resolved without a
/// separate lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionMatrix {
    pub groups: Vec<PermissionGroup>,
    /// Column roles as `(role_id, rank)`, ordered most- to least-powerful.
    pub roles: Vec<(RoleId, i16)>,
    pub cells: Vec<MatrixCell>,
}

impl PermissionMatrix {
    /// The raw stored state for a specific `(action, role)` cell, if present.
    #[must_use]
    pub fn raw_state(&self, action_key: &str, role: RoleId) -> Option<PermissionState> {
        self.cells
            .iter()
            .find(|c| c.action_key == action_key && c.role_id == role)
            .map(|c| c.state)
    }

    fn rank_of(&self, role: RoleId) -> Option<i16> {
        self.roles
            .iter()
            .find(|(id, _)| *id == role)
            .map(|(_, r)| *r)
    }

    /// The **effective** state for `(action, role)` after applying downward
    /// inheritance: a role gets the most permissive state among its own cell and
    /// every column role ranked below it. Returns `None` only when the action is
    /// unknown to every role (deny-by-default is applied by the caller).
    #[must_use]
    pub fn effective_state(&self, action_key: &str, role: RoleId) -> Option<PermissionState> {
        let my_rank = self.rank_of(role)?;
        let mut best: Option<PermissionState> = None;
        for (rid, rank) in &self.roles {
            // self, plus every column role at or below this role's rank
            if *rank <= my_rank {
                if let Some(state) = self.raw_state(action_key, *rid) {
                    best = Some(best.map_or(state, |b| b.max(state)));
                }
            }
        }
        best
    }

    /// All actions flattened, in matrix order.
    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.groups.iter().flat_map(|g| g.actions.iter())
    }
}
