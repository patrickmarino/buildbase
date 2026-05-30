//! Repository traits (data-access ports). All async via `async-trait` so they
//! remain object-safe and can be stored as `Arc<dyn ...Repo>` in the app layer.

use super::RepoResult;
use crate::entities::email::Email;
use crate::entities::{
    ApiKey, ApiKeyStatus, AuditEvent, Organization, PermissionMatrix, PermissionState, Role,
    Session, User, UserStatus,
};
use crate::ids::{ApiKeyId, OrgId, RoleId, SessionId, UserId};
use async_trait::async_trait;

/// Filter for listing users (mirrors the Users page toolbar).
#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    pub role_id: Option<RoleId>,
    pub status: Option<UserStatus>,
    /// Case-insensitive substring over name + email.
    pub query: Option<String>,
}

/// Query for searching the audit log (mirrors the Audit page toolbar).
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub category: Option<crate::entities::PermissionCategory>,
    /// Case-insensitive substring over actor + action + target.
    pub query: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> RepoResult<Option<User>>;
    async fn find_by_email(&self, org: OrgId, email: &Email) -> RepoResult<Option<User>>;
    async fn list(&self, org: OrgId, filter: &UserFilter) -> RepoResult<Vec<User>>;
    async fn insert(&self, user: &User) -> RepoResult<()>;
    async fn update(&self, user: &User) -> RepoResult<()>;
    /// Count active users whose role is Owner or Admin — drives last-admin protection.
    async fn count_active_admins(&self, org: OrgId) -> RepoResult<u64>;
}

#[async_trait]
pub trait RoleRepo: Send + Sync {
    async fn find_by_id(&self, id: RoleId) -> RepoResult<Option<Role>>;
    async fn find_by_key(&self, org: OrgId, key: &str) -> RepoResult<Option<Role>>;
    /// All roles for the org (columns + guest/service), ordered most→least powerful.
    async fn list_all(&self, org: OrgId) -> RepoResult<Vec<Role>>;
    async fn insert(&self, role: &Role) -> RepoResult<()>;
}

#[async_trait]
pub trait PermissionRepo: Send + Sync {
    /// Load the full matrix (groups, column roles + ranks, cells) for an org.
    async fn load_matrix(&self, org: OrgId) -> RepoResult<PermissionMatrix>;
    /// Upsert a single cell's state.
    async fn set_cell(
        &self,
        org: OrgId,
        action_key: &str,
        role: RoleId,
        state: PermissionState,
    ) -> RepoResult<()>;
    /// Insert a column role and copy every cell from `base` into `new_role`.
    async fn add_role_with_copied_cells(
        &self,
        org: OrgId,
        new_role: RoleId,
        base: RoleId,
    ) -> RepoResult<()>;
}

#[async_trait]
pub trait OrgRepo: Send + Sync {
    async fn get(&self, org: OrgId) -> RepoResult<Option<Organization>>;
    async fn update(&self, org: &Organization) -> RepoResult<()>;
    /// Atomically complete an ownership transfer: promote `new_owner` to the
    /// owner role, demote the previous owner to admin, set `owner_id`, and clear
    /// `pending_owner_id`.
    async fn complete_ownership_transfer(
        &self,
        org: OrgId,
        new_owner: UserId,
        previous_owner: UserId,
        owner_role: RoleId,
        admin_role: RoleId,
    ) -> RepoResult<()>;
    /// Permanently delete the organization and all dependent rows.
    async fn delete(&self, org: OrgId) -> RepoResult<()>;
}

#[async_trait]
pub trait AuditRepo: Send + Sync {
    /// Append a new event (append-only — never updated or deleted in code).
    async fn append(&self, event: &AuditEvent) -> RepoResult<()>;
    async fn search(&self, org: OrgId, query: &AuditQuery) -> RepoResult<Vec<AuditEvent>>;
}

#[async_trait]
pub trait ApiKeyRepo: Send + Sync {
    async fn list(&self, org: OrgId) -> RepoResult<Vec<ApiKey>>;
    async fn find_by_id(&self, id: ApiKeyId) -> RepoResult<Option<ApiKey>>;
    async fn insert(&self, key: &ApiKey) -> RepoResult<()>;
    async fn set_status(&self, id: ApiKeyId, status: ApiKeyStatus) -> RepoResult<()>;
}

#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn create(&self, session: &Session) -> RepoResult<()>;
    async fn get(&self, id: &SessionId) -> RepoResult<Option<Session>>;
    async fn delete(&self, id: &SessionId) -> RepoResult<()>;
    /// Revoke every session for a user — used on deactivate and role change.
    async fn delete_all_for_user(&self, user: UserId) -> RepoResult<()>;
}
