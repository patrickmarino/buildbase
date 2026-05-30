//! Authentication: password login, session creation, and resolving the acting
//! principal from a session id (the cookie value).

use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::email::Email;
use core_domain::entities::{PasswordResetToken, Session, User, UserStatus};
use core_domain::error::DomainError;
use core_domain::ids::{OrgId, PasswordResetTokenId, SessionId};
use core_domain::ports::{
    Clock, EmailSender, InviteTokenRepo, OrgRepo, OutgoingEmail, PasswordHasher,
    PasswordResetTokenRepo, PermissionRepo, RoleRepo, SessionRepo, TokenGenerator, UserRepo,
};
use core_domain::services::password_policy;
use std::sync::Arc;
use time::Duration;

/// How long a password-reset link stays valid.
const RESET_TTL: Duration = Duration::hours(1);

#[derive(Clone)]
pub struct AuthService {
    users: Arc<dyn UserRepo>,
    roles: Arc<dyn RoleRepo>,
    permissions: Arc<dyn PermissionRepo>,
    sessions: Arc<dyn SessionRepo>,
    orgs: Arc<dyn OrgRepo>,
    invite_tokens: Arc<dyn InviteTokenRepo>,
    reset_tokens: Arc<dyn PasswordResetTokenRepo>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenGenerator>,
    mailer: Arc<dyn EmailSender>,
    /// Base URL of the SPA — used to build the reset-password link.
    app_base_url: String,
    clock: Arc<dyn Clock>,
    session_ttl: Duration,
}

impl AuthService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        users: Arc<dyn UserRepo>,
        roles: Arc<dyn RoleRepo>,
        permissions: Arc<dyn PermissionRepo>,
        sessions: Arc<dyn SessionRepo>,
        orgs: Arc<dyn OrgRepo>,
        invite_tokens: Arc<dyn InviteTokenRepo>,
        reset_tokens: Arc<dyn PasswordResetTokenRepo>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenGenerator>,
        mailer: Arc<dyn EmailSender>,
        app_base_url: String,
        clock: Arc<dyn Clock>,
        session_ttl: Duration,
    ) -> Self {
        Self {
            users,
            roles,
            permissions,
            sessions,
            orgs,
            invite_tokens,
            reset_tokens,
            hasher,
            tokens,
            mailer,
            app_base_url,
            clock,
            session_ttl,
        }
    }

    /// Accept an invitation: exchange a valid token for a password + an active
    /// account, and start a session (so the user is signed in). Invalid/expired
    /// tokens and already-accepted invites are rejected; the password must satisfy
    /// the org policy.
    pub async fn accept_invite(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<(User, Session), AppError> {
        let hash = self.tokens.hash_api_token(token);
        let invite = self
            .invite_tokens
            .find_by_hash(&hash)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let now = self.clock.now();
        if invite.is_expired(now) {
            let _ = self.invite_tokens.delete(invite.id).await;
            return Err(AppError::Unauthorized);
        }

        let mut user = self
            .users
            .find_by_id(invite.user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        if user.status != UserStatus::Invited {
            return Err(DomainError::Invalid("this invitation is no longer valid".into()).into());
        }

        let org = self
            .orgs
            .get(user.org_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("organization".into()))?;
        password_policy::validate_password(new_password, &org.password_policy)
            .map_err(DomainError::PasswordPolicy)?;

        user.password_hash = Some(self.hasher.hash(new_password)?);
        user.status = UserStatus::Active;
        user.last_active_at = Some(now);
        self.users.update(&user).await?;
        // Consume every outstanding token for this user.
        self.invite_tokens.delete_for_user(user.id).await?;

        let session = Session {
            id: self.tokens.new_session_id(),
            user_id: user.id,
            created_at: now,
            expires_at: now + self.session_ttl,
        };
        self.sessions.create(&session).await?;
        Ok((user, session))
    }

    /// Begin a self-serve password reset. Always succeeds from the caller's view
    /// (and the HTTP layer always returns the same response) so an attacker can't
    /// probe which emails exist. When the email matches an **active** user we mint
    /// a single-use, short-lived token and email the reset link.
    pub async fn request_password_reset(&self, org: OrgId, email: &str) -> Result<(), AppError> {
        // A malformed email simply matches no user — no error leaked.
        let Ok(email) = Email::parse(email) else {
            return Ok(());
        };
        let Some(user) = self.users.find_by_email(org, &email).await? else {
            return Ok(());
        };
        // Only active accounts reset a password (invited users use accept-invite;
        // deactivated users cannot sign in at all).
        if user.status != UserStatus::Active {
            return Ok(());
        }

        let full = self.tokens.new_password_reset_token();
        let now = self.clock.now();
        // Invalidate any prior outstanding tokens, then store only the hash.
        let _ = self.reset_tokens.delete_for_user(user.id).await;
        self.reset_tokens
            .insert(&PasswordResetToken {
                id: PasswordResetTokenId::new(),
                user_id: user.id,
                token_hash: self.tokens.hash_api_token(&full),
                created_at: now,
                expires_at: now + RESET_TTL,
            })
            .await?;

        let org_name = self
            .orgs
            .get(user.org_id)
            .await
            .ok()
            .flatten()
            .map(|o| o.name)
            .unwrap_or_else(|| "the studio".into());
        let reset_url = format!(
            "{}/reset-password?token={full}",
            self.app_base_url.trim_end_matches('/')
        );
        // Best-effort: a delivery failure never fails the request (and never
        // reveals to the caller whether an email was actually sent).
        if let Err(e) = self
            .mailer
            .send(&OutgoingEmail {
                to_address: user.email.to_string(),
                to_name: user.name.clone(),
                subject: format!("Reset your {org_name} password"),
                body: format!(
                    "Hi {name},\n\nWe received a request to reset your {org_name} password. \
                     Choose a new one here:\n\n{reset_url}\n\nThis link expires in 1 hour. If you \
                     didn't request this, you can safely ignore this email.\n\n— {org_name}",
                    name = user.name,
                ),
            })
            .await
        {
            tracing::warn!("password-reset email to {} failed: {e}", user.email);
        }
        Ok(())
    }

    /// Complete a password reset: exchange a valid token for a new password,
    /// revoke every existing session (defence in depth), and start a fresh one so
    /// the user is signed in. Invalid/expired tokens are rejected; the new
    /// password must satisfy the org policy.
    pub async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<(User, Session), AppError> {
        let hash = self.tokens.hash_api_token(token);
        let reset = self
            .reset_tokens
            .find_by_hash(&hash)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let now = self.clock.now();
        if reset.is_expired(now) {
            let _ = self.reset_tokens.delete(reset.id).await;
            return Err(AppError::Unauthorized);
        }

        let mut user = self
            .users
            .find_by_id(reset.user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        // The token outlived the account becoming usable for sign-in.
        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized);
        }

        let org = self
            .orgs
            .get(user.org_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("organization".into()))?;
        password_policy::validate_password(new_password, &org.password_policy)
            .map_err(DomainError::PasswordPolicy)?;

        user.password_hash = Some(self.hasher.hash(new_password)?);
        user.last_active_at = Some(now);
        self.users.update(&user).await?;
        // Consume the reset token and revoke any sessions opened before the reset.
        self.reset_tokens.delete_for_user(user.id).await?;
        self.sessions.delete_all_for_user(user.id).await?;

        let session = Session {
            id: self.tokens.new_session_id(),
            user_id: user.id,
            created_at: now,
            expires_at: now + self.session_ttl,
        };
        self.sessions.create(&session).await?;
        Ok((user, session))
    }

    /// Verify credentials and start a session. All failure modes collapse to
    /// `Unauthorized` so we never reveal whether the email exists.
    pub async fn login(
        &self,
        org: OrgId,
        email: &str,
        password: &str,
    ) -> Result<(core_domain::entities::User, Session), AppError> {
        let email = Email::parse(email).map_err(|_| AppError::Unauthorized)?;
        let user = self
            .users
            .find_by_email(org, &email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let Some(hash) = user.password_hash.as_deref() else {
            return Err(AppError::Unauthorized);
        };
        if !self.hasher.verify(password, hash)? {
            return Err(AppError::Unauthorized);
        }
        if !user.is_active() {
            return Err(AppError::Unauthorized);
        }

        let now = self.clock.now();
        let session = Session {
            id: self.tokens.new_session_id(),
            user_id: user.id,
            created_at: now,
            expires_at: now + self.session_ttl,
        };
        self.sessions.create(&session).await?;
        Ok((user, session))
    }

    /// Resolve the actor for a session id. Expired sessions are deleted;
    /// deactivated users are rejected. Loads the role + matrix so downstream
    /// authorization is synchronous.
    pub async fn resolve_actor(
        &self,
        session_id: &SessionId,
        ip: Option<String>,
    ) -> Result<ActorContext, AppError> {
        let session = self
            .sessions
            .get(session_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if session.is_expired(self.clock.now()) {
            let _ = self.sessions.delete(session_id).await;
            return Err(AppError::Unauthorized);
        }

        let user = self
            .users
            .find_by_id(session.user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        if !user.is_active() {
            return Err(AppError::Unauthorized);
        }

        let actor_role = self
            .roles
            .find_by_id(user.role_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        let matrix = self.permissions.load_matrix(user.org_id).await?;

        Ok(ActorContext {
            actor: user,
            actor_role,
            matrix,
            ip,
        })
    }

    /// End a session. Idempotent.
    pub async fn logout(&self, session_id: &SessionId) -> Result<(), AppError> {
        self.sessions.delete(session_id).await?;
        Ok(())
    }
}
