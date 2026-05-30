//! Organization use-cases: read/update settings, ownership transfer (two-step:
//! initiate then accept), and deletion. All mutations are audited.

use crate::audit::Auditor;
use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::organization::{MfaConfig, PasswordPolicy, SsoConfig};
use core_domain::entities::{Organization, PermissionCategory};
use core_domain::error::DomainError;
use core_domain::ids::UserId;
use core_domain::ports::{OrgRepo, RoleRepo, UserRepo};
use std::sync::Arc;

/// A partial update of organization settings. Only `Some` fields are applied.
#[derive(Debug, Clone, Default)]
pub struct OrgSettingsPatch {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub accent_color: Option<String>,
    pub mfa: Option<MfaConfig>,
    pub password_policy: Option<PasswordPolicy>,
    pub sso: Option<SsoConfig>,
}

#[derive(Clone)]
pub struct OrgService {
    orgs: Arc<dyn OrgRepo>,
    users: Arc<dyn UserRepo>,
    roles: Arc<dyn RoleRepo>,
    auditor: Auditor,
}

impl OrgService {
    pub fn new(
        orgs: Arc<dyn OrgRepo>,
        users: Arc<dyn UserRepo>,
        roles: Arc<dyn RoleRepo>,
        auditor: Auditor,
    ) -> Self {
        Self { orgs, users, roles, auditor }
    }

    pub async fn get(&self, ctx: &ActorContext) -> Result<Organization, AppError> {
        ctx.require("org.branding")?;
        self.load(ctx).await
    }

    /// Apply a settings patch. Requires the auth-config permission.
    pub async fn update(
        &self,
        ctx: &ActorContext,
        patch: OrgSettingsPatch,
    ) -> Result<Organization, AppError> {
        ctx.require("org.auth")?;

        let mut org = self.load(ctx).await?;
        let before = summarize(&org);

        if let Some(name) = patch.name {
            org.name = name;
        }
        if let Some(domain) = patch.domain {
            org.domain = if domain.trim().is_empty() { None } else { Some(domain) };
        }
        if let Some(accent) = patch.accent_color {
            org.branding.accent_color = accent;
        }
        if let Some(mfa) = patch.mfa {
            org.mfa = mfa;
        }
        if let Some(pp) = patch.password_policy {
            org.password_policy = pp;
        }
        if let Some(sso) = patch.sso {
            org.sso = sso;
        }

        self.orgs.update(&org).await?;
        self.auditor
            .record(
                ctx,
                "org.settings.update",
                PermissionCategory::Org,
                Some("Organization settings".into()),
                Some(before),
                Some(summarize(&org)),
            )
            .await?;
        Ok(org)
    }

    /// Initiate an ownership transfer to an active admin. Sets `pending_owner_id`;
    /// the target must accept before it takes effect.
    pub async fn transfer_ownership(
        &self,
        ctx: &ActorContext,
        target: UserId,
    ) -> Result<(), AppError> {
        ctx.require("org.transfer")?;

        let mut org = self.load(ctx).await?;
        let candidate = self
            .users
            .find_by_id(target)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("user {target}")))?;
        if !candidate.is_active() {
            return Err(DomainError::Invalid("target user is not active".into()).into());
        }

        org.pending_owner_id = Some(target);
        self.orgs.update(&org).await?;

        self.auditor
            .record(
                ctx,
                "org.ownership.transfer",
                PermissionCategory::Org,
                Some(candidate.name.clone()),
                Some("Admin".into()),
                Some("Owner (pending accept)".into()),
            )
            .await?;
        Ok(())
    }

    /// Accept a pending ownership transfer. The actor must be the pending owner.
    /// Promotes them to Owner and demotes the previous owner to Admin atomically.
    pub async fn accept_ownership(&self, ctx: &ActorContext) -> Result<(), AppError> {
        let org = self.load(ctx).await?;
        if org.pending_owner_id != Some(ctx.actor.id) {
            return Err(DomainError::Forbidden("no pending transfer for you to accept".into()).into());
        }

        let owner_role = self.role_by_key(ctx, "owner").await?;
        let admin_role = self.role_by_key(ctx, "admin").await?;

        self.orgs
            .complete_ownership_transfer(org.id, ctx.actor.id, org.owner_id, owner_role.id, admin_role.id)
            .await?;

        self.auditor
            .record(
                ctx,
                "org.ownership.accept",
                PermissionCategory::Org,
                Some(ctx.actor.name.clone()),
                Some("Admin".into()),
                Some("Owner".into()),
            )
            .await?;
        Ok(())
    }

    /// Delete the organization. Owner-only (enforced by the matrix lock + authz).
    pub async fn delete_org(&self, ctx: &ActorContext) -> Result<(), AppError> {
        ctx.require("org.delete")?;
        let org = self.load(ctx).await?;
        // Record before deletion (the row is cascade-removed with the org).
        let _ = self
            .auditor
            .record(
                ctx,
                "org.delete",
                PermissionCategory::Org,
                Some(org.name.clone()),
                Some("Active".into()),
                Some("Deleted".into()),
            )
            .await;
        self.orgs.delete(org.id).await?;
        Ok(())
    }

    // ── helpers ───────────────────────────────────────────────
    async fn load(&self, ctx: &ActorContext) -> Result<Organization, AppError> {
        self.orgs
            .get(ctx.actor.org_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("organization".into()).into())
    }
    async fn role_by_key(&self, ctx: &ActorContext, key: &str) -> Result<core_domain::entities::Role, AppError> {
        self.roles
            .find_by_key(ctx.actor.org_id, key)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("role '{key}'")).into())
    }
}

/// A compact human summary used as the audit before/after snapshot.
fn summarize(org: &Organization) -> String {
    format!(
        "MFA {} · pw min {} · SSO {}",
        if org.mfa.enabled { "on" } else { "off" },
        org.password_policy.min_length,
        if org.sso.enabled { org.sso.provider.serialize_str() } else { "off" },
    )
}

trait SerializeStr {
    fn serialize_str(&self) -> &'static str;
}
impl SerializeStr for core_domain::entities::organization::SsoProvider {
    fn serialize_str(&self) -> &'static str {
        match self {
            core_domain::entities::organization::SsoProvider::Saml => "saml",
            core_domain::entities::organization::SsoProvider::Oidc => "oidc",
        }
    }
}
