//! The shared application state: all use-case services, wired over a Postgres
//! pool, plus config and the default organization id (single-tenant dev).

use crate::config::WebConfig;
use core_app::audit::Auditor;
use core_app::{
    ApiKeyService, AuditService, AuthService, OrgService, PermissionService, RoleService,
    UserService,
};
use core_domain::ids::OrgId;
use core_domain::ports::{
    ApiKeyRepo, AuditRepo, Clock, EmailSender, InviteTokenRepo, OrgRepo, PasswordHasher,
    PermissionRepo, RoleRepo, SessionRepo, TokenGenerator, UserRepo,
};
use core_infra::{
    Argon2Hasher, NoopMailer, PgApiKeyRepo, PgAuditRepo, PgInviteTokenRepo, PgOrgRepo,
    PgPermissionRepo, PgRoleRepo, PgSessionRepo, PgUserRepo, RandTokenGenerator, SmtpMailer,
    SystemClock,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub users: Arc<UserService>,
    pub roles: Arc<RoleService>,
    pub permissions: Arc<PermissionService>,
    pub org: Arc<OrgService>,
    pub audit: Arc<AuditService>,
    pub keys: Arc<ApiKeyService>,
    pub cfg: WebConfig,
    /// The organization sign-ins resolve to (single-tenant dev deployment).
    pub default_org: OrgId,
}

/// Build the configured mailer: SMTP (Mailpit in dev) when `SMTP_HOST` is set,
/// else a logging no-op.
pub fn default_mailer(cfg: &WebConfig) -> Arc<dyn EmailSender> {
    match cfg.smtp_host.as_deref() {
        Some(host) => {
            match SmtpMailer::new(host, cfg.smtp_port, &cfg.email_from, &cfg.email_from_name) {
                Ok(m) => Arc::new(m),
                Err(e) => {
                    tracing::warn!("email disabled: {e}");
                    Arc::new(NoopMailer)
                }
            }
        }
        None => Arc::new(NoopMailer),
    }
}

impl AppState {
    pub fn new(pool: PgPool, cfg: WebConfig, default_org: OrgId) -> Self {
        let mailer = default_mailer(&cfg);
        Self::with_mailer(pool, cfg, default_org, mailer)
    }

    /// Like [`Self::new`] but with an injected mailer (used by tests to capture mail).
    pub fn with_mailer(
        pool: PgPool,
        cfg: WebConfig,
        default_org: OrgId,
        mailer: Arc<dyn EmailSender>,
    ) -> Self {
        let users_repo: Arc<dyn UserRepo> = Arc::new(PgUserRepo::new(pool.clone()));
        let roles_repo: Arc<dyn RoleRepo> = Arc::new(PgRoleRepo::new(pool.clone()));
        let perms_repo: Arc<dyn PermissionRepo> = Arc::new(PgPermissionRepo::new(pool.clone()));
        let org_repo: Arc<dyn OrgRepo> = Arc::new(PgOrgRepo::new(pool.clone()));
        let audit_repo: Arc<dyn AuditRepo> = Arc::new(PgAuditRepo::new(pool.clone()));
        let keys_repo: Arc<dyn ApiKeyRepo> = Arc::new(PgApiKeyRepo::new(pool.clone()));
        let sessions_repo: Arc<dyn SessionRepo> = Arc::new(PgSessionRepo::new(pool.clone()));
        let invite_tokens_repo: Arc<dyn InviteTokenRepo> =
            Arc::new(PgInviteTokenRepo::new(pool.clone()));

        let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2Hasher);
        let tokens: Arc<dyn TokenGenerator> = Arc::new(RandTokenGenerator);
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let auditor = Auditor::new(audit_repo.clone(), clock.clone());

        let auth = Arc::new(AuthService::new(
            users_repo.clone(),
            roles_repo.clone(),
            perms_repo.clone(),
            sessions_repo.clone(),
            org_repo.clone(),
            invite_tokens_repo.clone(),
            hasher.clone(),
            tokens.clone(),
            clock.clone(),
            cfg.session_ttl,
        ));
        let users = Arc::new(UserService::new(
            users_repo.clone(),
            roles_repo.clone(),
            sessions_repo.clone(),
            org_repo.clone(),
            invite_tokens_repo.clone(),
            hasher.clone(),
            tokens.clone(),
            mailer.clone(),
            cfg.web_origin.clone(),
            auditor.clone(),
            clock.clone(),
        ));
        let roles = Arc::new(RoleService::new(
            roles_repo.clone(),
            perms_repo.clone(),
            auditor.clone(),
        ));
        let permissions = Arc::new(PermissionService::new(
            perms_repo.clone(),
            roles_repo.clone(),
            auditor.clone(),
        ));
        let org = Arc::new(OrgService::new(
            org_repo.clone(),
            users_repo.clone(),
            roles_repo.clone(),
            auditor.clone(),
        ));
        let audit = Arc::new(AuditService::new(audit_repo.clone()));
        let keys = Arc::new(ApiKeyService::new(keys_repo, tokens, clock, auditor));

        Self {
            auth,
            users,
            roles,
            permissions,
            org,
            audit,
            keys,
            cfg,
            default_org,
        }
    }
}
