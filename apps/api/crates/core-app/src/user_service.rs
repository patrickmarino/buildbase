//! User identity use-cases: list, invite, change role, activate/deactivate.
//! Enforces no-privilege-escalation and last-admin protection, and revokes
//! sessions when access is reduced.

use crate::audit::Auditor;
use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::email::Email;
use core_domain::entities::role::BuiltinRole;
use core_domain::entities::{PermissionCategory, Role, User, UserStatus};
use core_domain::error::DomainError;
use core_domain::ids::{RoleId, UserId};
use core_domain::ports::{RoleRepo, SessionRepo, UserFilter, UserRepo};
use core_domain::services::role_guards;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    users: Arc<dyn UserRepo>,
    roles: Arc<dyn RoleRepo>,
    sessions: Arc<dyn SessionRepo>,
    auditor: Auditor,
    clock: Arc<dyn core_domain::ports::Clock>,
}

/// A role is "admin-level" if it is the builtin Owner or Admin — the roles that
/// last-admin protection counts.
fn is_admin_role(role: &Role) -> bool {
    matches!(role.builtin, Some(BuiltinRole::Owner | BuiltinRole::Admin))
}

impl UserService {
    pub fn new(
        users: Arc<dyn UserRepo>,
        roles: Arc<dyn RoleRepo>,
        sessions: Arc<dyn SessionRepo>,
        auditor: Auditor,
        clock: Arc<dyn core_domain::ports::Clock>,
    ) -> Self {
        Self { users, roles, sessions, auditor, clock }
    }

    pub async fn list(&self, ctx: &ActorContext, filter: UserFilter) -> Result<Vec<User>, AppError> {
        ctx.require("users.view")?;
        Ok(self.users.list(ctx.actor.org_id, &filter).await?)
    }

    /// Invite a user by email. Default role is Member; status starts Invited.
    pub async fn invite(
        &self,
        ctx: &ActorContext,
        email: &str,
        role_key: &str,
        scope: Option<String>,
    ) -> Result<User, AppError> {
        ctx.require("users.invite")?;

        let email = Email::parse(email)?;
        if self.users.find_by_email(ctx.actor.org_id, &email).await?.is_some() {
            return Err(DomainError::Conflict("a user with that email already exists".into()).into());
        }

        let role_key = if role_key.is_empty() { "member" } else { role_key };
        let role = self.role_by_key(ctx, role_key).await?;
        // Cannot invite someone at a role above your own.
        role_guards::check_no_privilege_escalation(&ctx.actor_role, &role)?;

        let name = derive_name(&email);
        let user = User {
            id: UserId::new(),
            org_id: ctx.actor.org_id,
            email,
            name,
            role_id: role.id,
            status: UserStatus::Invited,
            scope: scope.clone(),
            password_hash: None,
            created_at: self.clock.now(),
            last_active_at: None,
        };
        self.users.insert(&user).await?;

        self.auditor
            .record(
                ctx,
                "user.invite",
                PermissionCategory::Users,
                Some(user.email.to_string()),
                Some("—".into()),
                Some(format!("{} · {}", role.name, scope.unwrap_or_else(|| "—".into()))),
            )
            .await?;
        Ok(user)
    }

    /// Change a user's role. Blocks privilege escalation and last-admin removal,
    /// and revokes the target's sessions so the new permissions take effect.
    pub async fn change_role(
        &self,
        ctx: &ActorContext,
        target_id: UserId,
        new_role_id: RoleId,
    ) -> Result<User, AppError> {
        ctx.require("roles.assign")?;

        let mut target = self.user_by_id(target_id).await?;
        let current_role = self.role_by_id(target.role_id).await?;
        let new_role = self.role_by_id(new_role_id).await?;

        role_guards::check_no_privilege_escalation(&ctx.actor_role, &new_role)?;

        let removes_admin = is_admin_role(&current_role) && !is_admin_role(&new_role);
        if removes_admin {
            let count = self.users.count_active_admins(ctx.actor.org_id).await?;
            role_guards::check_last_admin(count, true)?;
        }

        if target.role_id == new_role_id {
            return Ok(target); // no-op
        }

        target.role_id = new_role_id;
        self.users.update(&target).await?;
        self.sessions.delete_all_for_user(target_id).await?;

        self.auditor
            .record(
                ctx,
                "role.assign",
                PermissionCategory::Roles,
                Some(target.name.clone()),
                Some(current_role.name.clone()),
                Some(new_role.name.clone()),
            )
            .await?;
        Ok(target)
    }

    /// Set a user's status (deactivate / reactivate). Deactivation revokes all of
    /// the user's sessions immediately, and is blocked if it would remove the last
    /// administrator.
    pub async fn set_status(
        &self,
        ctx: &ActorContext,
        target_id: UserId,
        status: UserStatus,
    ) -> Result<User, AppError> {
        ctx.require("users.deactivate")?;

        let mut target = self.user_by_id(target_id).await?;
        let role = self.role_by_id(target.role_id).await?;

        if status == UserStatus::Deactivated && is_admin_role(&role) {
            let count = self.users.count_active_admins(ctx.actor.org_id).await?;
            role_guards::check_last_admin(count, true)?;
        }

        let before = target.status;
        target.status = status;
        if status == UserStatus::Active {
            target.last_active_at = Some(self.clock.now());
        }
        self.users.update(&target).await?;

        if status == UserStatus::Deactivated {
            self.sessions.delete_all_for_user(target_id).await?;
        }

        let action = if status == UserStatus::Deactivated {
            "user.deactivate"
        } else {
            "user.reactivate"
        };
        self.auditor
            .record(
                ctx,
                action,
                PermissionCategory::Users,
                Some(target.name.clone()),
                Some(before.as_str().to_string()),
                Some(status.as_str().to_string()),
            )
            .await?;
        Ok(target)
    }

    // ── helpers ───────────────────────────────────────────────
    async fn user_by_id(&self, id: UserId) -> Result<User, AppError> {
        self.users
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("user {id}")).into())
    }
    async fn role_by_id(&self, id: RoleId) -> Result<Role, AppError> {
        self.roles
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("role {id}")).into())
    }
    async fn role_by_key(&self, ctx: &ActorContext, key: &str) -> Result<Role, AppError> {
        self.roles
            .find_by_key(ctx.actor.org_id, key)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("role '{key}'")).into())
    }
}

/// Derive a display name from an email local part, e.g. `aoife.brennan` → `Aoife Brennan`.
fn derive_name(email: &Email) -> String {
    let local = email.local_part().replace(['.', '_'], " ");
    local
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
