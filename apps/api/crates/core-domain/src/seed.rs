//! The canonical default dataset — the seven roles, the action catalog, and the
//! default permission states — transcribed from the prototype's `data.jsx`
//! (`SEED_ROLES` / `SEED_GROUPS`). This is the **single source of truth** for
//! seeding: both the infrastructure startup seed and the in-memory test fakes
//! build from it, so the two never drift.

use crate::entities::permission::{Action, PermissionCategory, PermissionGroup, PermissionState};
use crate::entities::role::BuiltinRole;

use PermissionState::{Allow as A, Deny as D, Scope as S};

/// One row of the default matrix: the action plus its default state for each of
/// the five column roles, in order: owner, admin, manager, member, viewer.
struct Row {
    key: &'static str,
    category: PermissionCategory,
    label: &'static str,
    states: [PermissionState; 5],
}

const COLUMN_ROLES: [BuiltinRole; 5] = [
    BuiltinRole::Owner,
    BuiltinRole::Admin,
    BuiltinRole::Manager,
    BuiltinRole::Member,
    BuiltinRole::Viewer,
];

#[rustfmt::skip]
const ROWS: &[Row] = &[
    // Users
    Row { key: "users.invite",     category: PermissionCategory::Users, label: "Invite user",                 states: [A, A, S, D, D] },
    Row { key: "users.view",       category: PermissionCategory::Users, label: "View user list",              states: [A, A, S, D, D] },
    Row { key: "users.edit",       category: PermissionCategory::Users, label: "Edit user profile",           states: [A, A, S, S, S] },
    Row { key: "users.deactivate", category: PermissionCategory::Users, label: "Deactivate user",             states: [A, A, S, D, D] },
    // Roles & permissions
    Row { key: "roles.edit",       category: PermissionCategory::Roles, label: "Create / edit role",          states: [A, A, D, D, D] },
    Row { key: "roles.assign",     category: PermissionCategory::Roles, label: "Assign role",                 states: [A, S, S, D, D] },
    Row { key: "roles.owner",      category: PermissionCategory::Roles, label: "Grant Owner",                 states: [A, D, D, D, D] },
    Row { key: "roles.matrix",     category: PermissionCategory::Roles, label: "View permission matrix",      states: [A, A, A, D, D] },
    // Organization settings
    Row { key: "org.branding",     category: PermissionCategory::Org,   label: "Edit org profile / branding", states: [A, A, D, D, D] },
    Row { key: "org.auth",         category: PermissionCategory::Org,   label: "Configure auth (MFA, SSO, policy)", states: [A, A, D, D, D] },
    Row { key: "org.transfer",     category: PermissionCategory::Org,   label: "Transfer ownership",          states: [A, D, D, D, D] },
    Row { key: "org.delete",       category: PermissionCategory::Org,   label: "Delete organization",         states: [A, D, D, D, D] },
    // Audit log
    Row { key: "audit.view",       category: PermissionCategory::Audit, label: "View audit log",              states: [A, A, S, D, D] },
    Row { key: "audit.export",     category: PermissionCategory::Audit, label: "Export audit log",            states: [A, A, D, D, D] },
    // Service accounts & API keys
    Row { key: "keys.manage",      category: PermissionCategory::Keys,  label: "Create / revoke key",         states: [A, A, D, D, D] },
    Row { key: "keys.scope",       category: PermissionCategory::Keys,  label: "Scope key permissions",       states: [A, A, D, D, D] },
];

const CATEGORY_ORDER: [PermissionCategory; 5] = [
    PermissionCategory::Users,
    PermissionCategory::Roles,
    PermissionCategory::Org,
    PermissionCategory::Audit,
    PermissionCategory::Keys,
];

/// The action catalog as ordered groups (matrix rows).
#[must_use]
pub fn default_groups() -> Vec<PermissionGroup> {
    CATEGORY_ORDER
        .iter()
        .map(|&cat| PermissionGroup {
            category: cat,
            actions: ROWS
                .iter()
                .filter(|r| r.category == cat)
                .map(|r| Action {
                    key: r.key.to_string(),
                    category: r.category,
                    label: r.label.to_string(),
                })
                .collect(),
        })
        .collect()
}

/// The default cell states as `(action_key, role_key, state)` triples for every
/// column role.
#[must_use]
pub fn default_cells() -> Vec<(&'static str, &'static str, PermissionState)> {
    let mut cells = Vec::new();
    for row in ROWS {
        for (i, role) in COLUMN_ROLES.iter().enumerate() {
            cells.push((row.key, role.key(), row.states[i]));
        }
    }
    cells
}

/// The default state for one `(action, column role)` pair, if defined.
#[must_use]
pub fn default_state(action_key: &str, role: BuiltinRole) -> Option<PermissionState> {
    let idx = COLUMN_ROLES.iter().position(|&r| r == role)?;
    ROWS.iter()
        .find(|r| r.key == action_key)
        .map(|r| r.states[idx])
}

/// Default organization settings (branding accent, MFA, password policy, SSO),
/// matching the prototype's initial Organization state.
#[must_use]
pub fn default_branding_accent() -> &'static str {
    "#D6B982"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_five_cells() {
        let cells = default_cells();
        assert_eq!(cells.len(), ROWS.len() * 5);
    }

    #[test]
    fn groups_cover_all_actions() {
        let total: usize = default_groups().iter().map(|g| g.actions.len()).sum();
        assert_eq!(total, ROWS.len());
    }

    #[test]
    fn owner_allows_everything() {
        for row in ROWS {
            assert_eq!(
                default_state(row.key, BuiltinRole::Owner),
                Some(PermissionState::Allow)
            );
        }
    }

    #[test]
    fn owner_only_actions_deny_admin() {
        for key in crate::services::matrix_rules::OWNER_ONLY_ACTIONS {
            assert_eq!(
                default_state(key, BuiltinRole::Admin),
                Some(PermissionState::Deny)
            );
        }
    }
}
