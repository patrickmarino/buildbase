//! Permission-matrix editing rules: how a cell cycles, and which cells are
//! locked by policy. Mirrors `roles.jsx`/`data.jsx` in the prototype.

use crate::entities::permission::{MatrixCell, PermissionMatrix, PermissionState};
use crate::error::DomainError;
use crate::ids::RoleId;

/// Actions that only the Owner may ever hold — every non-owner cell is locked at
/// Deny, and the Owner cell is locked at Allow. From the PRD's ACL matrix.
pub const OWNER_ONLY_ACTIONS: [&str; 3] = ["roles.owner", "org.transfer", "org.delete"];

/// The next state when a cell is clicked: Allow → Scope → Deny → Allow.
#[must_use]
pub fn cycle(state: PermissionState) -> PermissionState {
    match state {
        PermissionState::Allow => PermissionState::Scope,
        PermissionState::Scope => PermissionState::Deny,
        PermissionState::Deny => PermissionState::Allow,
    }
}

/// Whether the cell for `(action_key, role_key)` is locked (cannot be edited).
#[must_use]
pub fn is_locked(action_key: &str, role_key: &str) -> bool {
    lock_reason(action_key, role_key).is_some()
}

/// The reason a cell is locked, or `None` if it is editable. The reason strings
/// match the prototype's hover copy.
#[must_use]
pub fn lock_reason(action_key: &str, role_key: &str) -> Option<&'static str> {
    if role_key == "owner" {
        return Some("Owner has full control by definition.");
    }
    if OWNER_ONLY_ACTIONS.contains(&action_key) {
        return Some(match action_key {
            "roles.owner" => "Only the Owner may grant ownership.",
            _ => "Owner-only action.",
        });
    }
    None
}

/// Apply a click to the matrix: cycle the `(action_key, role)` cell in place and
/// return its new value. Errors with [`DomainError::CellLocked`] if the cell is
/// locked by rule, or [`DomainError::NotFound`] if the cell does not exist.
pub fn apply_click(
    matrix: &mut PermissionMatrix,
    action_key: &str,
    role: RoleId,
    role_key: &str,
) -> Result<MatrixCell, DomainError> {
    if let Some(reason) = lock_reason(action_key, role_key) {
        return Err(DomainError::CellLocked(reason.to_string()));
    }
    let cell = matrix
        .cells
        .iter_mut()
        .find(|c| c.action_key == action_key && c.role_id == role)
        .ok_or_else(|| {
            DomainError::NotFound(format!("cell {action_key} for role {role} not found"))
        })?;
    cell.state = cycle(cell.state);
    Ok(cell.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::permission::{Action, PermissionCategory, PermissionGroup};

    fn role_id(n: u128) -> RoleId {
        RoleId::from_uuid(uuid::Uuid::from_u128(n))
    }

    fn test_matrix() -> (PermissionMatrix, RoleId, RoleId) {
        let owner = role_id(1);
        let admin = role_id(2);
        let groups = vec![PermissionGroup {
            category: PermissionCategory::Users,
            actions: vec![Action {
                key: "users.invite".into(),
                category: PermissionCategory::Users,
                label: "Invite user".into(),
            }],
        }];
        let cells = vec![
            MatrixCell { action_key: "users.invite".into(), role_id: owner, state: PermissionState::Allow },
            MatrixCell { action_key: "users.invite".into(), role_id: admin, state: PermissionState::Allow },
            MatrixCell { action_key: "org.delete".into(), role_id: admin, state: PermissionState::Deny },
        ];
        let matrix = PermissionMatrix {
            groups,
            roles: vec![(owner, 60), (admin, 50)],
            cells,
        };
        (matrix, owner, admin)
    }

    #[test]
    fn cycle_wraps_allow_scope_deny() {
        assert_eq!(cycle(PermissionState::Allow), PermissionState::Scope);
        assert_eq!(cycle(PermissionState::Scope), PermissionState::Deny);
        assert_eq!(cycle(PermissionState::Deny), PermissionState::Allow);
    }

    #[test]
    fn owner_column_is_locked() {
        assert!(is_locked("users.invite", "owner"));
        assert_eq!(lock_reason("users.invite", "owner"), Some("Owner has full control by definition."));
    }

    #[test]
    fn owner_only_actions_locked_for_non_owner() {
        assert!(is_locked("roles.owner", "admin"));
        assert!(is_locked("org.transfer", "admin"));
        assert!(is_locked("org.delete", "manager"));
        assert_eq!(lock_reason("roles.owner", "admin"), Some("Only the Owner may grant ownership."));
    }

    #[test]
    fn ordinary_cell_is_editable() {
        assert!(!is_locked("users.invite", "admin"));
        assert_eq!(lock_reason("users.invite", "admin"), None);
    }

    #[test]
    fn apply_click_cycles_editable_cell() {
        let (mut m, _owner, admin) = test_matrix();
        let cell = apply_click(&mut m, "users.invite", admin, "admin").unwrap();
        assert_eq!(cell.state, PermissionState::Scope);
        // persisted in the matrix
        assert_eq!(m.raw_state("users.invite", admin), Some(PermissionState::Scope));
    }

    #[test]
    fn apply_click_rejects_owner_column() {
        let (mut m, owner, _admin) = test_matrix();
        let err = apply_click(&mut m, "users.invite", owner, "owner").unwrap_err();
        assert!(matches!(err, DomainError::CellLocked(_)));
        // unchanged
        assert_eq!(m.raw_state("users.invite", owner), Some(PermissionState::Allow));
    }

    #[test]
    fn apply_click_rejects_owner_only_action() {
        let (mut m, _owner, admin) = test_matrix();
        let err = apply_click(&mut m, "org.delete", admin, "admin").unwrap_err();
        assert!(matches!(err, DomainError::CellLocked(_)));
        assert_eq!(m.raw_state("org.delete", admin), Some(PermissionState::Deny));
    }

    #[test]
    fn apply_click_missing_cell_errors() {
        let (mut m, _owner, admin) = test_matrix();
        let err = apply_click(&mut m, "nonexistent.action", admin, "admin").unwrap_err();
        assert!(matches!(err, DomainError::NotFound(_)));
    }
}
