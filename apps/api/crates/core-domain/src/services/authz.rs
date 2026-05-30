//! The central authorization decision (PRD FR-4): `can(actor, action, resource)`,
//! **deny-by-default**. Every other module trusts this — it is the substrate for
//! "who can do what". Pure: it reads an already-loaded [`PermissionMatrix`].

use crate::entities::permission::{PermissionMatrix, PermissionState};
use crate::ids::{RoleId, UserId};

/// The outcome of an authorization check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

impl Decision {
    #[must_use]
    pub fn is_allowed(self) -> bool {
        matches!(self, Decision::Allow)
    }
}

/// Everything `can` needs to decide. Borrowed so the call allocates nothing.
pub struct AuthzInput<'a> {
    /// The acting principal's primary role.
    pub actor_role: RoleId,
    /// The org's permission matrix.
    pub matrix: &'a PermissionMatrix,
    /// The acting principal's user id (for `scope` = self-owned decisions).
    pub actor_id: UserId,
    /// The owner of the resource being acted on, when relevant. `None` for
    /// collection-level or non-owned actions (then `scope` resolves to deny).
    pub resource_owner: Option<UserId>,
}

/// Decide whether the actor may perform `action`.
///
/// - Unknown action / no cell ⇒ **Deny** (deny-by-default).
/// - `Allow` ⇒ Allow.
/// - `Deny` ⇒ Deny.
/// - `Scope` ⇒ Allow only when acting on the actor's own resource, else Deny.
///
/// Inheritance is applied: a higher-ranked role gets the most permissive state
/// among itself and all roles below it (see [`PermissionMatrix::effective_state`]).
#[must_use]
pub fn can(input: &AuthzInput, action: &str) -> Decision {
    match input.matrix.effective_state(action, input.actor_role) {
        Some(PermissionState::Allow) => Decision::Allow,
        Some(PermissionState::Scope) => {
            if input.resource_owner.is_some() && input.resource_owner == Some(input.actor_id) {
                Decision::Allow
            } else {
                Decision::Deny
            }
        }
        Some(PermissionState::Deny) | None => Decision::Deny,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::permission::{Action, MatrixCell, PermissionCategory, PermissionGroup};

    fn rid(n: u128) -> RoleId {
        RoleId::from_uuid(uuid::Uuid::from_u128(n))
    }
    fn uid(n: u128) -> UserId {
        UserId::from_uuid(uuid::Uuid::from_u128(n))
    }

    /// owner(rank 60), admin(50), member(30) with a couple of actions.
    fn matrix(cells: Vec<MatrixCell>) -> PermissionMatrix {
        PermissionMatrix {
            groups: vec![PermissionGroup {
                category: PermissionCategory::Users,
                actions: vec![Action {
                    key: "users.invite".into(),
                    category: PermissionCategory::Users,
                    label: "Invite".into(),
                }],
            }],
            roles: vec![(rid(1), 60), (rid(2), 50), (rid(3), 30)],
            cells,
        }
    }

    fn cell(action: &str, role: RoleId, state: PermissionState) -> MatrixCell {
        MatrixCell {
            action_key: action.into(),
            role_id: role,
            state,
        }
    }

    #[test]
    fn deny_by_default_for_unknown_action() {
        let m = matrix(vec![]);
        let input = AuthzInput {
            actor_role: rid(2),
            matrix: &m,
            actor_id: uid(99),
            resource_owner: None,
        };
        assert_eq!(can(&input, "nonexistent"), Decision::Deny);
    }

    #[test]
    fn allow_when_cell_allows() {
        let m = matrix(vec![cell("users.invite", rid(2), PermissionState::Allow)]);
        let input = AuthzInput {
            actor_role: rid(2),
            matrix: &m,
            actor_id: uid(99),
            resource_owner: None,
        };
        assert_eq!(can(&input, "users.invite"), Decision::Allow);
    }

    #[test]
    fn deny_when_cell_denies() {
        let m = matrix(vec![cell("users.invite", rid(3), PermissionState::Deny)]);
        let input = AuthzInput {
            actor_role: rid(3),
            matrix: &m,
            actor_id: uid(99),
            resource_owner: None,
        };
        assert_eq!(can(&input, "users.invite"), Decision::Deny);
    }

    #[test]
    fn scope_allows_self_owned_only() {
        let m = matrix(vec![cell("users.invite", rid(3), PermissionState::Scope)]);
        let me = uid(7);
        // acting on my own resource → allow
        let mine = AuthzInput {
            actor_role: rid(3),
            matrix: &m,
            actor_id: me,
            resource_owner: Some(me),
        };
        assert_eq!(can(&mine, "users.invite"), Decision::Allow);
        // acting on someone else's → deny
        let other = AuthzInput {
            actor_role: rid(3),
            matrix: &m,
            actor_id: me,
            resource_owner: Some(uid(8)),
        };
        assert_eq!(can(&other, "users.invite"), Decision::Deny);
        // collection-level (no owner) → deny
        let none = AuthzInput {
            actor_role: rid(3),
            matrix: &m,
            actor_id: me,
            resource_owner: None,
        };
        assert_eq!(can(&none, "users.invite"), Decision::Deny);
    }

    #[test]
    fn higher_role_inherits_lower_allow() {
        // admin allows; owner has no explicit cell but should inherit allow.
        let m = matrix(vec![cell("users.invite", rid(2), PermissionState::Allow)]);
        let owner = AuthzInput {
            actor_role: rid(1),
            matrix: &m,
            actor_id: uid(1),
            resource_owner: None,
        };
        assert_eq!(can(&owner, "users.invite"), Decision::Allow);
        // member (below admin) does NOT inherit upward.
        let member = AuthzInput {
            actor_role: rid(3),
            matrix: &m,
            actor_id: uid(3),
            resource_owner: None,
        };
        assert_eq!(can(&member, "users.invite"), Decision::Deny);
    }

    #[test]
    fn inheritance_takes_most_permissive() {
        // member=scope, admin=deny: owner inherits the most permissive (scope),
        // which on a self-owned resource resolves to allow.
        let m = matrix(vec![
            cell("users.invite", rid(3), PermissionState::Scope),
            cell("users.invite", rid(2), PermissionState::Deny),
        ]);
        let me = uid(1);
        let owner = AuthzInput {
            actor_role: rid(1),
            matrix: &m,
            actor_id: me,
            resource_owner: Some(me),
        };
        assert_eq!(can(&owner, "users.invite"), Decision::Allow);
    }
}
