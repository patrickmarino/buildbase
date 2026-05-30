//! Guardrails around role changes: no privilege escalation, last-admin
//! protection, and custom-role inheritance. From the PRD §6.6 edge cases.

use crate::entities::permission::MatrixCell;
use crate::entities::role::Role;
use crate::error::DomainError;
use crate::ids::RoleId;

/// A user may never be assigned a role ranked **above** the actor's own role.
/// Equal rank is permitted (an Admin may assign Admin). The Owner (highest rank)
/// can assign anything, including Owner.
pub fn check_no_privilege_escalation(actor: &Role, target_new: &Role) -> Result<(), DomainError> {
    if target_new.rank > actor.rank {
        Err(DomainError::PrivilegeEscalation(format!(
            "cannot assign '{}' (rank {}) — above your own role '{}' (rank {})",
            target_new.key, target_new.rank, actor.key, actor.rank
        )))
    } else {
        Ok(())
    }
}

/// Last-admin protection: the final active Owner/Admin cannot be removed or
/// demoted. `removes_admin` is true when the pending change (deactivation, or a
/// demotion below admin level) would drop an admin; `active_admin_count` is the
/// current number of active Owner+Admin users.
pub fn check_last_admin(active_admin_count: u64, removes_admin: bool) -> Result<(), DomainError> {
    if removes_admin && active_admin_count <= 1 {
        Err(DomainError::LastAdminProtected(
            "the organization must retain at least one active administrator".into(),
        ))
    } else {
        Ok(())
    }
}

/// Build the matrix cells for a new custom role by copying the base role's cells,
/// re-pointed at `new_role`. The new role "starts with everything the chosen role
/// can do — then you tune it in the matrix" (prototype `NewRoleModal`).
#[must_use]
pub fn build_custom_role_cells(base_cells: &[MatrixCell], new_role: RoleId) -> Vec<MatrixCell> {
    base_cells
        .iter()
        .map(|c| MatrixCell {
            action_key: c.action_key.clone(),
            role_id: new_role,
            state: c.state,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::permission::PermissionState;
    use crate::entities::role::BuiltinRole;
    use crate::ids::OrgId;

    fn role(key: &str, rank: i16) -> Role {
        Role {
            id: RoleId::new(),
            org_id: OrgId::new(),
            key: key.into(),
            name: key.into(),
            builtin: BuiltinRole::from_key(key),
            is_column: true,
            base_role_id: None,
            rank,
        }
    }

    #[test]
    fn cannot_assign_above_own_rank() {
        let admin = role("admin", 50);
        let owner = role("owner", 60);
        assert!(check_no_privilege_escalation(&admin, &owner).is_err());
    }

    #[test]
    fn can_assign_at_or_below_own_rank() {
        let admin = role("admin", 50);
        let member = role("member", 30);
        let admin2 = role("admin", 50);
        assert!(check_no_privilege_escalation(&admin, &member).is_ok());
        assert!(check_no_privilege_escalation(&admin, &admin2).is_ok());
    }

    #[test]
    fn owner_can_assign_owner() {
        let owner = role("owner", 60);
        let owner_target = role("owner", 60);
        assert!(check_no_privilege_escalation(&owner, &owner_target).is_ok());
    }

    #[test]
    fn last_admin_cannot_be_removed() {
        assert!(check_last_admin(1, true).is_err());
        assert!(matches!(
            check_last_admin(1, true).unwrap_err(),
            DomainError::LastAdminProtected(_)
        ));
    }

    #[test]
    fn admin_removable_when_others_remain() {
        assert!(check_last_admin(2, true).is_ok());
    }

    #[test]
    fn non_admin_change_never_triggers_protection() {
        // Deactivating a non-admin (removes_admin = false) is always fine.
        assert!(check_last_admin(1, false).is_ok());
    }

    #[test]
    fn custom_role_copies_base_cells() {
        let base = RoleId::new();
        let new = RoleId::new();
        let base_cells = vec![
            MatrixCell {
                action_key: "users.invite".into(),
                role_id: base,
                state: PermissionState::Allow,
            },
            MatrixCell {
                action_key: "org.delete".into(),
                role_id: base,
                state: PermissionState::Deny,
            },
        ];
        let copied = build_custom_role_cells(&base_cells, new);
        assert_eq!(copied.len(), 2);
        assert!(copied.iter().all(|c| c.role_id == new));
        assert_eq!(copied[0].action_key, "users.invite");
        assert_eq!(copied[0].state, PermissionState::Allow);
        assert_eq!(copied[1].state, PermissionState::Deny);
    }
}
