//! The acting principal's context, resolved once per request from the session.
//! Carries the matrix so authorization is a pure, synchronous check.

use crate::error::AppError;
use core_domain::entities::{PermissionMatrix, Role, User};
use core_domain::error::DomainError;
use core_domain::ids::UserId;
use core_domain::services::authz::{can, AuthzInput};

#[derive(Debug, Clone)]
pub struct ActorContext {
    pub actor: User,
    pub actor_role: Role,
    /// The org's permission matrix, loaded at auth time so `require` is sync.
    pub matrix: PermissionMatrix,
    /// Client IP for audit records.
    pub ip: Option<String>,
}

impl ActorContext {
    /// Capability check: may the actor perform `action` at all? `Allow` and
    /// `Scope` both pass — the precise limit of a `Scope` permission (own level,
    /// own team, self only) is enforced by the dedicated domain guards
    /// (privilege-escalation, last-admin) or by [`Self::require_on`]. Only a
    /// `Deny`/missing cell is forbidden here.
    pub fn require(&self, action: &str) -> Result<(), AppError> {
        // Evaluate "as self" so a `Scope` cell resolves to Allow.
        self.require_on(action, Some(self.actor.id))
    }

    /// Strict check: authorize `action` scoped to the owner of the target
    /// resource, so a `Scope` permission resolves to allow only on the actor's
    /// own data. Use for genuinely self-only operations.
    pub fn require_on(&self, action: &str, resource_owner: Option<UserId>) -> Result<(), AppError> {
        let input = AuthzInput {
            actor_role: self.actor_role.id,
            matrix: &self.matrix,
            actor_id: self.actor.id,
            resource_owner,
        };
        if can(&input, action).is_allowed() {
            Ok(())
        } else {
            Err(AppError::Domain(DomainError::Forbidden(format!(
                "role '{}' may not {action}",
                self.actor_role.key
            ))))
        }
    }
}
