//! The permission matrix use-cases: read the matrix, and cycle a cell's state
//! (the centerpiece interaction). Every edit is audited.

use crate::audit::Auditor;
use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::{MatrixCell, PermissionCategory, PermissionMatrix, PermissionState};
use core_domain::error::DomainError;
use core_domain::ids::RoleId;
use core_domain::ports::{PermissionRepo, RoleRepo};
use core_domain::services::matrix_rules;
use std::sync::Arc;

#[derive(Clone)]
pub struct PermissionService {
    permissions: Arc<dyn PermissionRepo>,
    roles: Arc<dyn RoleRepo>,
    auditor: Auditor,
}

impl PermissionService {
    pub fn new(
        permissions: Arc<dyn PermissionRepo>,
        roles: Arc<dyn RoleRepo>,
        auditor: Auditor,
    ) -> Self {
        Self { permissions, roles, auditor }
    }

    pub async fn get_matrix(&self, ctx: &ActorContext) -> Result<PermissionMatrix, AppError> {
        ctx.require("roles.matrix")?;
        Ok(self.permissions.load_matrix(ctx.actor.org_id).await?)
    }

    /// Cycle a cell allow → scope → deny. Rejects locked cells; persists and
    /// audits the change. Returns the new cell.
    pub async fn cycle_cell(
        &self,
        ctx: &ActorContext,
        action_key: &str,
        role_id: RoleId,
    ) -> Result<MatrixCell, AppError> {
        ctx.require("roles.edit")?;

        let role = self
            .roles
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("role {role_id}")))?;

        let mut matrix = self.permissions.load_matrix(ctx.actor.org_id).await?;
        let before = matrix.raw_state(action_key, role_id).unwrap_or(PermissionState::Deny);

        // Pure rule: enforces lock + cycles in the loaded matrix.
        let cell = matrix_rules::apply_click(&mut matrix, action_key, role_id, &role.key)?;

        self.permissions
            .set_cell(ctx.actor.org_id, action_key, role_id, cell.state)
            .await?;

        let action_label = matrix
            .actions()
            .find(|a| a.key == action_key)
            .map(|a| a.label.clone())
            .unwrap_or_else(|| action_key.to_string());

        self.auditor
            .record(
                ctx,
                "permission.edit",
                PermissionCategory::Roles,
                Some(format!("{} · {}", role.name, action_label)),
                Some(before.label().to_string()),
                Some(cell.state.label().to_string()),
            )
            .await?;
        Ok(cell)
    }
}
