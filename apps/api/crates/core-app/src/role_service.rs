//! Role use-cases: listing roles and creating custom roles that inherit from a
//! base role.

use crate::audit::Auditor;
use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::Role;
use core_domain::entities::PermissionCategory;
use core_domain::error::DomainError;
use core_domain::ids::RoleId;
use core_domain::ports::{PermissionRepo, RoleRepo};
use std::sync::Arc;

#[derive(Clone)]
pub struct RoleService {
    roles: Arc<dyn RoleRepo>,
    permissions: Arc<dyn PermissionRepo>,
    auditor: Auditor,
}

impl RoleService {
    pub fn new(
        roles: Arc<dyn RoleRepo>,
        permissions: Arc<dyn PermissionRepo>,
        auditor: Auditor,
    ) -> Self {
        Self { roles, permissions, auditor }
    }

    pub async fn list(&self, ctx: &ActorContext) -> Result<Vec<Role>, AppError> {
        ctx.require("roles.matrix")?;
        Ok(self.roles.list_all(ctx.actor.org_id).await?)
    }

    /// Create a custom column role that copies the base role's permissions. The
    /// new role can never exceed the actor's own level.
    pub async fn create_custom(
        &self,
        ctx: &ActorContext,
        name: &str,
        base_role_id: RoleId,
    ) -> Result<Role, AppError> {
        ctx.require("roles.edit")?;

        let name = name.trim();
        if name.is_empty() {
            return Err(DomainError::Invalid("role name is required".into()).into());
        }
        let key = slugify(name);
        if self.roles.find_by_key(ctx.actor.org_id, &key).await?.is_some() {
            return Err(DomainError::Conflict("a role with that name already exists".into()).into());
        }

        let base = self
            .roles
            .find_by_id(base_role_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("base role {base_role_id}")))?;

        let role = Role {
            id: RoleId::new(),
            org_id: ctx.actor.org_id,
            key,
            name: name.to_string(),
            builtin: None,
            is_column: true,
            base_role_id: Some(base.id),
            // A custom role sits at its base's tier; it inherits everything below.
            rank: base.rank,
        };
        self.roles.insert(&role).await?;
        self.permissions
            .add_role_with_copied_cells(ctx.actor.org_id, role.id, base.id)
            .await?;

        self.auditor
            .record(
                ctx,
                "role.create",
                PermissionCategory::Roles,
                Some(format!("{} (custom)", role.name)),
                Some("—".into()),
                Some(format!("inherits {}", base.name)),
            )
            .await?;
        Ok(role)
    }
}

/// `"Stager Pro"` → `"stager_pro"`. Keeps `[a-z0-9_]`.
fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.trim().to_lowercase().chars() {
        if c.is_alphanumeric() {
            out.push(c);
        } else if (c.is_whitespace() || c == '-' || c == '_') && !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}
