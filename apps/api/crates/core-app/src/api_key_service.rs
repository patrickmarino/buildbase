//! Service-account API key use-cases: list, create (token shown once), revoke.

use crate::audit::Auditor;
use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::{ApiKey, ApiKeyStatus, PermissionCategory, Scope};
use core_domain::error::DomainError;
use core_domain::ids::{ApiKeyId, OrgId};
use core_domain::ports::{ApiKeyRepo, Clock, TokenGenerator};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiKeyService {
    keys: Arc<dyn ApiKeyRepo>,
    tokens: Arc<dyn TokenGenerator>,
    clock: Arc<dyn Clock>,
    auditor: Auditor,
}

/// The result of creating a key: the stored key plus the one-time plaintext token.
pub struct CreatedKey {
    pub key: ApiKey,
    /// Shown to the caller exactly once; never persisted in plaintext.
    pub token: String,
}

impl ApiKeyService {
    pub fn new(
        keys: Arc<dyn ApiKeyRepo>,
        tokens: Arc<dyn TokenGenerator>,
        clock: Arc<dyn Clock>,
        auditor: Auditor,
    ) -> Self {
        Self { keys, tokens, clock, auditor }
    }

    pub async fn list(&self, ctx: &ActorContext) -> Result<Vec<ApiKey>, AppError> {
        ctx.require("keys.scope")?;
        Ok(self.keys.list(ctx.actor.org_id).await?)
    }

    /// Create a service account. The full token is returned once; only its hash
    /// and display prefix are stored.
    pub async fn create(
        &self,
        ctx: &ActorContext,
        name: &str,
        scopes: Vec<Scope>,
    ) -> Result<CreatedKey, AppError> {
        ctx.require("keys.manage")?;

        let name = name.trim();
        if name.is_empty() {
            return Err(DomainError::Invalid("name is required".into()).into());
        }
        if scopes.is_empty() {
            return Err(DomainError::Invalid("select at least one scope".into()).into());
        }

        let token = self.tokens.new_api_token();
        let key = ApiKey {
            id: ApiKeyId::new(),
            org_id: ctx.actor.org_id,
            name: name.to_string(),
            prefix: token.prefix.clone(),
            token_hash: self.tokens.hash_api_token(&token.full),
            scopes: scopes.clone(),
            status: ApiKeyStatus::Active,
            created_at: self.clock.now(),
            last_used_at: None,
        };
        self.keys.insert(&key).await?;

        let scope_list = scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        self.auditor
            .record(
                ctx,
                "key.create",
                PermissionCategory::Keys,
                Some(name.to_string()),
                Some("—".into()),
                Some(format!("scopes: {scope_list}")),
            )
            .await?;

        Ok(CreatedKey { key, token: token.full })
    }

    pub async fn revoke(&self, ctx: &ActorContext, id: ApiKeyId) -> Result<(), AppError> {
        ctx.require("keys.manage")?;

        let key = self
            .find_in_org(ctx.actor.org_id, id)
            .await?;
        self.keys.set_status(id, ApiKeyStatus::Revoked).await?;

        self.auditor
            .record(
                ctx,
                "key.revoke",
                PermissionCategory::Keys,
                Some(key.name.clone()),
                Some("Active".into()),
                Some("Revoked".into()),
            )
            .await?;
        Ok(())
    }

    async fn find_in_org(&self, org: OrgId, id: ApiKeyId) -> Result<ApiKey, AppError> {
        let key = self
            .keys
            .find_by_id(id)
            .await?
            .filter(|k| k.org_id == org)
            .ok_or_else(|| DomainError::NotFound(format!("api key {id}")))?;
        Ok(key)
    }
}
