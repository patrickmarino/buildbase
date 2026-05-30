//! Authentication: password login, session creation, and resolving the acting
//! principal from a session id (the cookie value).

use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::email::Email;
use core_domain::entities::{Session, User, UserStatus};
use core_domain::error::DomainError;
use core_domain::ids::{OrgId, SessionId};
use core_domain::ports::{
    Clock, InviteTokenRepo, OrgRepo, PasswordHasher, PermissionRepo, RoleRepo, SessionRepo,
    TokenGenerator, UserRepo,
};
use core_domain::services::password_policy;
use std::sync::Arc;
use time::Duration;

#[derive(Clone)]
pub struct AuthService {
    users: Arc<dyn UserRepo>,
    roles: Arc<dyn RoleRepo>,
    permissions: Arc<dyn PermissionRepo>,
    sessions: Arc<dyn SessionRepo>,
    orgs: Arc<dyn OrgRepo>,
    invite_tokens: Arc<dyn InviteTokenRepo>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenGenerator>,
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
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenGenerator>,
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
            hasher,
            tokens,
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
