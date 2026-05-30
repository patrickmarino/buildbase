//! In-memory fakes for the domain ports, plus a [`World`] that wires a seeded
//! org, roles, matrix, and a few users into ready-to-use services. Used by the
//! use-case tests so they run with no database.

// Test scaffolding: a few ergonomic shortcuts are fine here.
#![allow(
    clippy::unwrap_used,
    clippy::too_many_arguments,
    clippy::unnecessary_sort_by
)]

use crate::audit::Auditor;
use crate::ctx::ActorContext;
use core_domain::entities::email::Email;
use core_domain::entities::organization::{
    Branding, MfaConfig, MfaEnforce, MfaMethod, PasswordPolicy, SsoConfig, SsoProvider,
};
use core_domain::entities::role::BuiltinRole;
use core_domain::entities::{
    ApiKey, ApiKeyStatus, AuditEvent, Organization, PermissionMatrix, PermissionState, Role,
    Session, User, UserStatus,
};
use core_domain::ids::*;
use core_domain::ports::*;
use core_domain::seed;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use time::{Duration, OffsetDateTime};

// ── Shared store ──────────────────────────────────────────────
/// One store, many repo views. The repo fakes are thin wrappers over an
/// `Arc<Store>` so e.g. `count_active_admins` can resolve role keys.
#[derive(Default)]
pub struct Store {
    pub users: Mutex<HashMap<UserId, User>>,
    pub roles: Mutex<HashMap<RoleId, Role>>,
    pub cells: Mutex<HashMap<(String, RoleId), PermissionState>>,
    pub org: Mutex<Option<Organization>>,
    pub audit: Mutex<Vec<AuditEvent>>,
    pub keys: Mutex<HashMap<ApiKeyId, ApiKey>>,
    pub sessions: Mutex<HashMap<SessionId, Session>>,
}

impl Store {
    fn column_roles_sorted(&self) -> Vec<(RoleId, i16)> {
        let roles = self.roles.lock().unwrap();
        let mut cols: Vec<(RoleId, i16)> = roles
            .values()
            .filter(|r| r.is_column)
            .map(|r| (r.id, r.rank))
            .collect();
        cols.sort_by(|a, b| b.1.cmp(&a.1));
        cols
    }
}

fn unique_violation(msg: &str) -> RepoError {
    RepoError::UniqueViolation(msg.to_string())
}

// ── UserRepo ──────────────────────────────────────────────────
pub struct UserRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl UserRepo for UserRepoFake {
    async fn find_by_id(&self, id: UserId) -> RepoResult<Option<User>> {
        Ok(self.0.users.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_email(&self, org: OrgId, email: &Email) -> RepoResult<Option<User>> {
        Ok(self
            .0
            .users
            .lock()
            .unwrap()
            .values()
            .find(|u| u.org_id == org && u.email == *email)
            .cloned())
    }
    async fn list(&self, org: OrgId, filter: &UserFilter) -> RepoResult<Vec<User>> {
        let q = filter.query.as_deref().map(str::to_lowercase);
        let mut out: Vec<User> = self
            .0
            .users
            .lock()
            .unwrap()
            .values()
            .filter(|u| u.org_id == org)
            .filter(|u| filter.role_id.is_none_or(|r| u.role_id == r))
            .filter(|u| filter.status.is_none_or(|s| u.status == s))
            .filter(|u| {
                q.as_ref().is_none_or(|q| {
                    u.name.to_lowercase().contains(q) || u.email.as_str().contains(q)
                })
            })
            .cloned()
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
    async fn insert(&self, user: &User) -> RepoResult<()> {
        let mut users = self.0.users.lock().unwrap();
        if users
            .values()
            .any(|u| u.org_id == user.org_id && u.email == user.email)
        {
            return Err(unique_violation("email"));
        }
        users.insert(user.id, user.clone());
        Ok(())
    }
    async fn update(&self, user: &User) -> RepoResult<()> {
        self.0.users.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }
    async fn count_active_admins(&self, org: OrgId) -> RepoResult<u64> {
        let roles = self.0.roles.lock().unwrap();
        let n = self
            .0
            .users
            .lock()
            .unwrap()
            .values()
            .filter(|u| u.org_id == org && u.status == UserStatus::Active)
            .filter(|u| {
                roles.get(&u.role_id).is_some_and(|r| {
                    matches!(r.builtin, Some(BuiltinRole::Owner | BuiltinRole::Admin))
                })
            })
            .count();
        Ok(n as u64)
    }
}

// ── RoleRepo ──────────────────────────────────────────────────
pub struct RoleRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl RoleRepo for RoleRepoFake {
    async fn find_by_id(&self, id: RoleId) -> RepoResult<Option<Role>> {
        Ok(self.0.roles.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_key(&self, org: OrgId, key: &str) -> RepoResult<Option<Role>> {
        Ok(self
            .0
            .roles
            .lock()
            .unwrap()
            .values()
            .find(|r| r.org_id == org && r.key == key)
            .cloned())
    }
    async fn list_all(&self, org: OrgId) -> RepoResult<Vec<Role>> {
        let mut out: Vec<Role> = self
            .0
            .roles
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.org_id == org)
            .cloned()
            .collect();
        out.sort_by(|a, b| b.rank.cmp(&a.rank).then(a.name.cmp(&b.name)));
        Ok(out)
    }
    async fn insert(&self, role: &Role) -> RepoResult<()> {
        let mut roles = self.0.roles.lock().unwrap();
        if roles
            .values()
            .any(|r| r.org_id == role.org_id && r.key == role.key)
        {
            return Err(unique_violation("role key"));
        }
        roles.insert(role.id, role.clone());
        Ok(())
    }
}

// ── PermissionRepo ────────────────────────────────────────────
pub struct PermissionRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl PermissionRepo for PermissionRepoFake {
    async fn load_matrix(&self, _org: OrgId) -> RepoResult<PermissionMatrix> {
        let roles = self.0.column_roles_sorted();
        let cells = self
            .0
            .cells
            .lock()
            .unwrap()
            .iter()
            .map(
                |((action, role), state)| core_domain::entities::MatrixCell {
                    action_key: action.clone(),
                    role_id: *role,
                    state: *state,
                },
            )
            .collect();
        Ok(PermissionMatrix {
            groups: seed::default_groups(),
            roles,
            cells,
        })
    }
    async fn set_cell(
        &self,
        _org: OrgId,
        action_key: &str,
        role: RoleId,
        state: PermissionState,
    ) -> RepoResult<()> {
        self.0
            .cells
            .lock()
            .unwrap()
            .insert((action_key.to_string(), role), state);
        Ok(())
    }
    async fn add_role_with_copied_cells(
        &self,
        _org: OrgId,
        new_role: RoleId,
        base: RoleId,
    ) -> RepoResult<()> {
        let mut cells = self.0.cells.lock().unwrap();
        let copies: Vec<((String, RoleId), PermissionState)> = cells
            .iter()
            .filter(|((_, r), _)| *r == base)
            .map(|((a, _), s)| ((a.clone(), new_role), *s))
            .collect();
        cells.extend(copies);
        Ok(())
    }
}

// ── OrgRepo ───────────────────────────────────────────────────
pub struct OrgRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl OrgRepo for OrgRepoFake {
    async fn get(&self, org: OrgId) -> RepoResult<Option<Organization>> {
        Ok(self.0.org.lock().unwrap().clone().filter(|o| o.id == org))
    }
    async fn update(&self, org: &Organization) -> RepoResult<()> {
        *self.0.org.lock().unwrap() = Some(org.clone());
        Ok(())
    }
    async fn complete_ownership_transfer(
        &self,
        _org: OrgId,
        new_owner: UserId,
        previous_owner: UserId,
        owner_role: RoleId,
        admin_role: RoleId,
    ) -> RepoResult<()> {
        {
            let mut users = self.0.users.lock().unwrap();
            if let Some(u) = users.get_mut(&new_owner) {
                u.role_id = owner_role;
            }
            if let Some(u) = users.get_mut(&previous_owner) {
                u.role_id = admin_role;
            }
        }
        let mut org = self.0.org.lock().unwrap();
        if let Some(o) = org.as_mut() {
            o.owner_id = new_owner;
            o.pending_owner_id = None;
        }
        Ok(())
    }
    async fn delete(&self, _org: OrgId) -> RepoResult<()> {
        *self.0.org.lock().unwrap() = None;
        Ok(())
    }
}

// ── AuditRepo ─────────────────────────────────────────────────
pub struct AuditRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl AuditRepo for AuditRepoFake {
    async fn append(&self, event: &AuditEvent) -> RepoResult<()> {
        self.0.audit.lock().unwrap().push(event.clone());
        Ok(())
    }
    async fn search(&self, org: OrgId, query: &AuditQuery) -> RepoResult<Vec<AuditEvent>> {
        let q = query.query.as_deref().map(str::to_lowercase);
        let mut out: Vec<AuditEvent> = self
            .0
            .audit
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.org_id == org)
            .filter(|e| query.category.is_none_or(|c| e.category == c))
            .filter(|e| {
                q.as_ref().is_none_or(|q| {
                    format!(
                        "{} {} {}",
                        e.actor_name,
                        e.action,
                        e.target.clone().unwrap_or_default()
                    )
                    .to_lowercase()
                    .contains(q)
                })
            })
            .cloned()
            .collect();
        out.sort_by(|a, b| b.ts.cmp(&a.ts));
        Ok(out)
    }
}

// ── ApiKeyRepo ────────────────────────────────────────────────
pub struct ApiKeyRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl ApiKeyRepo for ApiKeyRepoFake {
    async fn list(&self, org: OrgId) -> RepoResult<Vec<ApiKey>> {
        let mut out: Vec<ApiKey> = self
            .0
            .keys
            .lock()
            .unwrap()
            .values()
            .filter(|k| k.org_id == org)
            .cloned()
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(out)
    }
    async fn find_by_id(&self, id: ApiKeyId) -> RepoResult<Option<ApiKey>> {
        Ok(self.0.keys.lock().unwrap().get(&id).cloned())
    }
    async fn insert(&self, key: &ApiKey) -> RepoResult<()> {
        self.0.keys.lock().unwrap().insert(key.id, key.clone());
        Ok(())
    }
    async fn set_status(&self, id: ApiKeyId, status: ApiKeyStatus) -> RepoResult<()> {
        if let Some(k) = self.0.keys.lock().unwrap().get_mut(&id) {
            k.status = status;
        }
        Ok(())
    }
}

// ── SessionRepo ───────────────────────────────────────────────
pub struct SessionRepoFake(pub Arc<Store>);

#[async_trait::async_trait]
impl SessionRepo for SessionRepoFake {
    async fn create(&self, session: &Session) -> RepoResult<()> {
        self.0
            .sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session.clone());
        Ok(())
    }
    async fn get(&self, id: &SessionId) -> RepoResult<Option<Session>> {
        Ok(self.0.sessions.lock().unwrap().get(id).cloned())
    }
    async fn delete(&self, id: &SessionId) -> RepoResult<()> {
        self.0.sessions.lock().unwrap().remove(id);
        Ok(())
    }
    async fn delete_all_for_user(&self, user: UserId) -> RepoResult<()> {
        self.0
            .sessions
            .lock()
            .unwrap()
            .retain(|_, s| s.user_id != user);
        Ok(())
    }
}

// ── Fakes for the capability ports ────────────────────────────
pub struct FixedClock(pub OffsetDateTime);
impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.0
    }
}

/// Hash = `hashed::<pw>`; verify is exact-match. Deterministic, no crypto.
pub struct FakeHasher;
impl PasswordHasher for FakeHasher {
    fn hash(&self, plaintext: &str) -> Result<String, core_domain::error::DomainError> {
        Ok(format!("hashed::{plaintext}"))
    }
    fn verify(&self, plaintext: &str, phc: &str) -> Result<bool, core_domain::error::DomainError> {
        Ok(phc == format!("hashed::{plaintext}"))
    }
}

/// Monotonic, deterministic token/session generation for assertions.
#[derive(Default)]
pub struct FakeTokens {
    counter: AtomicU64,
}
impl TokenGenerator for FakeTokens {
    fn new_session_id(&self) -> SessionId {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        SessionId::new(format!("sess-{n}"))
    }
    fn new_api_token(&self) -> ApiToken {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let full = format!("ms_live_token{n:020}");
        ApiToken {
            prefix: full.chars().take(12).collect(),
            full,
        }
    }
    fn hash_api_token(&self, full: &str) -> String {
        format!("h::{full}")
    }
}

// ── World ─────────────────────────────────────────────────────
/// A fully-seeded in-memory world: one org, all default roles + matrix, an Owner,
/// an Admin, and a Member. Exposes the services and a context builder.
pub struct World {
    pub store: Arc<Store>,
    pub org_id: OrgId,
    pub roles_by_key: HashMap<String, RoleId>,
    pub owner_id: UserId,
    pub admin_id: UserId,
    pub member_id: UserId,
    pub auditor: Auditor,
    pub clock: Arc<dyn Clock>,
    pub tokens: Arc<FakeTokens>,
    pub hasher: Arc<FakeHasher>,
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        let store = Arc::new(Store::default());
        let org_id = OrgId::new();
        let now = OffsetDateTime::from_unix_timestamp(1_780_000_000).unwrap();

        // Roles
        let mut roles_by_key = HashMap::new();
        for b in BuiltinRole::ordered() {
            let id = RoleId::new();
            roles_by_key.insert(b.key().to_string(), id);
            store.roles.lock().unwrap().insert(
                id,
                Role {
                    id,
                    org_id,
                    key: b.key().to_string(),
                    name: b.display_name().to_string(),
                    builtin: Some(b),
                    is_column: b.is_column(),
                    base_role_id: None,
                    rank: b.rank(),
                },
            );
        }

        // Matrix cells
        for (action, role_key, state) in seed::default_cells() {
            let rid = roles_by_key[role_key];
            store
                .cells
                .lock()
                .unwrap()
                .insert((action.to_string(), rid), state);
        }

        // Users
        let owner_id = Self::insert_user(
            &store,
            org_id,
            "elena@madespace.co",
            "Elena Marchetti",
            roles_by_key["owner"],
            UserStatus::Active,
            Some("hashed::ownerpw"),
            now,
        );
        let admin_id = Self::insert_user(
            &store,
            org_id,
            "tomas@madespace.co",
            "Tomas Reinholt",
            roles_by_key["admin"],
            UserStatus::Active,
            Some("hashed::adminpw"),
            now,
        );
        let member_id = Self::insert_user(
            &store,
            org_id,
            "daniel@madespace.co",
            "Daniel Fischer",
            roles_by_key["member"],
            UserStatus::Active,
            Some("hashed::memberpw"),
            now,
        );

        // Org
        *store.org.lock().unwrap() = Some(Organization {
            id: org_id,
            name: "MadeSpace Studio".into(),
            domain: Some("madespace.co".into()),
            owner_id,
            pending_owner_id: None,
            branding: Branding {
                accent_color: seed::default_branding_accent().into(),
            },
            mfa: MfaConfig {
                enabled: true,
                method: MfaMethod::Totp,
                enforce: MfaEnforce::Admins,
            },
            password_policy: PasswordPolicy::default(),
            sso: SsoConfig {
                enabled: false,
                provider: SsoProvider::Saml,
                url: None,
            },
        });

        let clock: Arc<dyn Clock> = Arc::new(FixedClock(now));
        let auditor = Auditor::new(Arc::new(AuditRepoFake(store.clone())), clock.clone());

        Self {
            store,
            org_id,
            roles_by_key,
            owner_id,
            admin_id,
            member_id,
            auditor,
            clock,
            tokens: Arc::new(FakeTokens::default()),
            hasher: Arc::new(FakeHasher),
        }
    }

    fn insert_user(
        store: &Arc<Store>,
        org_id: OrgId,
        email: &str,
        name: &str,
        role_id: RoleId,
        status: UserStatus,
        hash: Option<&str>,
        now: OffsetDateTime,
    ) -> UserId {
        let id = UserId::new();
        store.users.lock().unwrap().insert(
            id,
            User {
                id,
                org_id,
                email: Email::parse(email).unwrap(),
                name: name.into(),
                role_id,
                status,
                scope: Some("Studio".into()),
                password_hash: hash.map(str::to_string),
                created_at: now,
                last_active_at: Some(now),
            },
        );
        id
    }

    // Repo view constructors
    pub fn users_repo(&self) -> Arc<dyn UserRepo> {
        Arc::new(UserRepoFake(self.store.clone()))
    }
    pub fn roles_repo(&self) -> Arc<dyn RoleRepo> {
        Arc::new(RoleRepoFake(self.store.clone()))
    }
    pub fn perms_repo(&self) -> Arc<dyn PermissionRepo> {
        Arc::new(PermissionRepoFake(self.store.clone()))
    }
    pub fn org_repo(&self) -> Arc<dyn OrgRepo> {
        Arc::new(OrgRepoFake(self.store.clone()))
    }
    pub fn keys_repo(&self) -> Arc<dyn ApiKeyRepo> {
        Arc::new(ApiKeyRepoFake(self.store.clone()))
    }
    pub fn sessions_repo(&self) -> Arc<dyn SessionRepo> {
        Arc::new(SessionRepoFake(self.store.clone()))
    }
    pub fn audit_repo(&self) -> Arc<dyn AuditRepo> {
        Arc::new(AuditRepoFake(self.store.clone()))
    }

    /// Build an [`ActorContext`] for a user, loading their role + the matrix.
    pub async fn ctx_for(&self, user_id: UserId) -> ActorContext {
        let actor = self
            .users_repo()
            .find_by_id(user_id)
            .await
            .unwrap()
            .unwrap();
        let actor_role = self
            .roles_repo()
            .find_by_id(actor.role_id)
            .await
            .unwrap()
            .unwrap();
        let matrix = self.perms_repo().load_matrix(self.org_id).await.unwrap();
        ActorContext {
            actor,
            actor_role,
            matrix,
            ip: Some("203.0.113.7".into()),
        }
    }

    /// Number of audit events recorded so far.
    pub fn audit_count(&self) -> usize {
        self.store.audit.lock().unwrap().len()
    }
    pub fn last_audit_action(&self) -> Option<String> {
        self.store
            .audit
            .lock()
            .unwrap()
            .last()
            .map(|e| e.action.clone())
    }
    pub fn session_count(&self) -> usize {
        self.store.sessions.lock().unwrap().len()
    }

    pub fn session_ttl() -> Duration {
        Duration::days(7)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
