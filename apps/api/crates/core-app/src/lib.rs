//! # core-app
//!
//! The application / use-case layer. Each service orchestrates the domain ports
//! (`Arc<dyn ...Repo>`) to fulfil a feature, enforcing two cross-cutting rules
//! uniformly:
//!
//! * **Authorization** — every guarded operation calls
//!   [`ctx::ActorContext::require`], which runs the pure deny-by-default
//!   [`core_domain::services::authz::can`] against the matrix loaded at auth time.
//! * **Auditing** — every mutation records an [`core_domain::entities::AuditEvent`]
//!   via [`audit::Auditor`], with before/after snapshots.
//!
//! Services depend only on the domain traits, so they are unit-tested against the
//! in-memory fakes in [`testing`] with no database.

pub mod audit;
pub mod ctx;
pub mod error;

pub mod api_key_service;
pub mod audit_service;
pub mod auth_service;
pub mod org_service;
pub mod permission_service;
pub mod role_service;
pub mod user_service;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;

#[cfg(test)]
mod usecase_tests;

pub use api_key_service::ApiKeyService;
pub use audit_service::AuditService;
pub use auth_service::AuthService;
pub use ctx::ActorContext;
pub use error::{AppError, AppResult};
pub use org_service::{OrgService, OrgSettingsPatch};
pub use permission_service::PermissionService;
pub use role_service::RoleService;
pub use user_service::UserService;
