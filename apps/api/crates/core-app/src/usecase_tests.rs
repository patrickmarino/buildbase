//! Use-case tests over the in-memory [`crate::testing`] world. These assert the
//! orchestration: authorization, guards, auditing, and session side effects —
//! the things the pure domain tests can't cover on their own.

use crate::testing::World;
use crate::*;
use core_domain::entities::{Scope, UserStatus};
use core_domain::error::DomainError;
use core_domain::ports::AuditQuery;

fn auth(w: &World) -> AuthService {
    AuthService::new(
        w.users_repo(),
        w.roles_repo(),
        w.perms_repo(),
        w.sessions_repo(),
        w.hasher.clone(),
        w.tokens.clone(),
        w.clock.clone(),
        World::session_ttl(),
    )
}
fn users(w: &World) -> UserService {
    UserService::new(w.users_repo(), w.roles_repo(), w.sessions_repo(), w.auditor.clone(), w.clock.clone())
}
fn perms(w: &World) -> PermissionService {
    PermissionService::new(w.perms_repo(), w.roles_repo(), w.auditor.clone())
}
fn roles(w: &World) -> RoleService {
    RoleService::new(w.roles_repo(), w.perms_repo(), w.auditor.clone())
}
fn org(w: &World) -> OrgService {
    OrgService::new(w.org_repo(), w.users_repo(), w.roles_repo(), w.auditor.clone())
}
fn audit(w: &World) -> AuditService {
    AuditService::new(w.audit_repo())
}
fn keys(w: &World) -> ApiKeyService {
    ApiKeyService::new(w.keys_repo(), w.tokens.clone(), w.clock.clone(), w.auditor.clone())
}

// ── Auth ──────────────────────────────────────────────────────
#[tokio::test]
async fn login_succeeds_with_correct_password() {
    let w = World::new();
    let (user, session) = auth(&w).login(w.org_id, "elena@madespace.co", "ownerpw").await.unwrap();
    assert_eq!(user.id, w.owner_id);
    assert_eq!(w.session_count(), 1);
    // session resolves back to the owner
    let ctx = auth(&w).resolve_actor(&session.id, None).await.unwrap();
    assert_eq!(ctx.actor.id, w.owner_id);
    assert_eq!(ctx.actor_role.key, "owner");
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let w = World::new();
    let err = auth(&w).login(w.org_id, "elena@madespace.co", "nope").await.unwrap_err();
    assert!(matches!(err, AppError::Unauthorized));
    assert_eq!(w.session_count(), 0);
}

#[tokio::test]
async fn login_rejects_deactivated_user() {
    let w = World::new();
    // deactivate the member first (as owner)
    users(&w).set_status(&w.ctx_for(w.owner_id).await, w.member_id, UserStatus::Deactivated).await.unwrap();
    let err = auth(&w).login(w.org_id, "daniel@madespace.co", "memberpw").await.unwrap_err();
    assert!(matches!(err, AppError::Unauthorized));
}

#[tokio::test]
async fn logout_invalidates_session() {
    let w = World::new();
    let (_u, s) = auth(&w).login(w.org_id, "elena@madespace.co", "ownerpw").await.unwrap();
    auth(&w).logout(&s.id).await.unwrap();
    assert!(auth(&w).resolve_actor(&s.id, None).await.is_err());
}

// ── Users ─────────────────────────────────────────────────────
#[tokio::test]
async fn invite_creates_invited_user_and_audits() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let before = w.audit_count();
    let u = users(&w).invite(&ctx, "aoife@madespace.co", "member", Some("Notting Hill".into())).await.unwrap();
    assert_eq!(u.status, UserStatus::Invited);
    assert_eq!(u.name, "Aoife");
    assert_eq!(w.audit_count(), before + 1);
    assert_eq!(w.last_audit_action().as_deref(), Some("user.invite"));
}

#[tokio::test]
async fn invite_rejects_duplicate_email() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let err = users(&w).invite(&ctx, "tomas@madespace.co", "member", None).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::Conflict(_))));
}

#[tokio::test]
async fn invite_rejects_invalid_email() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    assert!(users(&w).invite(&ctx, "not-an-email", "member", None).await.is_err());
}

#[tokio::test]
async fn member_cannot_invite() {
    let w = World::new();
    let ctx = w.ctx_for(w.member_id).await; // member: users.invite = deny
    let err = users(&w).invite(&ctx, "x@madespace.co", "member", None).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::Forbidden(_))));
}

#[tokio::test]
async fn change_role_revokes_sessions_and_audits_once() {
    let w = World::new();
    // give the member a live session
    let (_u, _s) = auth(&w).login(w.org_id, "daniel@madespace.co", "memberpw").await.unwrap();
    assert_eq!(w.session_count(), 1);

    let ctx = w.ctx_for(w.owner_id).await;
    let before = w.audit_count();
    let viewer_role = w.roles_by_key["viewer"];
    let updated = users(&w).change_role(&ctx, w.member_id, viewer_role).await.unwrap();
    assert_eq!(updated.role_id, viewer_role);
    assert_eq!(w.session_count(), 0, "target sessions revoked");
    assert_eq!(w.audit_count(), before + 1);
    assert_eq!(w.last_audit_action().as_deref(), Some("role.assign"));
}

#[tokio::test]
async fn last_admin_cannot_be_demoted() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    // deactivate the admin → owner becomes the sole admin
    users(&w).set_status(&ctx, w.admin_id, UserStatus::Deactivated).await.unwrap();
    let ctx = w.ctx_for(w.owner_id).await;
    // now demoting the owner to member would remove the last admin
    let member_role = w.roles_by_key["member"];
    let err = users(&w).change_role(&ctx, w.owner_id, member_role).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::LastAdminProtected(_))));
}

#[tokio::test]
async fn last_admin_cannot_be_deactivated() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    users(&w).set_status(&ctx, w.admin_id, UserStatus::Deactivated).await.unwrap();
    let ctx = w.ctx_for(w.owner_id).await;
    let err = users(&w).set_status(&ctx, w.owner_id, UserStatus::Deactivated).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::LastAdminProtected(_))));
}

#[tokio::test]
async fn deactivating_member_revokes_their_sessions() {
    let w = World::new();
    auth(&w).login(w.org_id, "daniel@madespace.co", "memberpw").await.unwrap();
    assert_eq!(w.session_count(), 1);
    let ctx = w.ctx_for(w.owner_id).await;
    users(&w).set_status(&ctx, w.member_id, UserStatus::Deactivated).await.unwrap();
    assert_eq!(w.session_count(), 0);
}

#[tokio::test]
async fn admin_cannot_assign_owner() {
    let w = World::new();
    let ctx = w.ctx_for(w.admin_id).await;
    let owner_role = w.roles_by_key["owner"];
    let err = users(&w).change_role(&ctx, w.member_id, owner_role).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::PrivilegeEscalation(_))));
}

// ── Permissions ───────────────────────────────────────────────
#[tokio::test]
async fn cycle_cell_changes_state_and_audits() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let admin_role = w.roles_by_key["admin"];
    // users.invite for admin starts Allow → cycles to Scope
    let cell = perms(&w).cycle_cell(&ctx, "users.invite", admin_role).await.unwrap();
    assert_eq!(cell.state, core_domain::entities::PermissionState::Scope);
    assert_eq!(w.last_audit_action().as_deref(), Some("permission.edit"));
}

#[tokio::test]
async fn cycle_cell_rejects_locked_cell() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let admin_role = w.roles_by_key["admin"];
    let before_audits = w.audit_count();
    // org.delete is owner-only → locked for admin
    let err = perms(&w).cycle_cell(&ctx, "org.delete", admin_role).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::CellLocked(_))));
    assert_eq!(w.audit_count(), before_audits, "nothing audited on a locked cell");
}

// ── Roles ─────────────────────────────────────────────────────
#[tokio::test]
async fn create_custom_role_copies_base_cells() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let viewer = w.roles_by_key["viewer"];
    let role = roles(&w).create_custom(&ctx, "Stager", viewer).await.unwrap();
    assert_eq!(role.key, "stager");
    assert_eq!(role.base_role_id, Some(viewer));

    // the new role should now be a column with copied cells
    let matrix = perms(&w).get_matrix(&w.ctx_for(w.owner_id).await).await.unwrap();
    assert!(matrix.roles.iter().any(|(id, _)| *id == role.id));
    // viewer denies users.invite → stager copied deny
    assert_eq!(
        matrix.raw_state("users.invite", role.id),
        Some(core_domain::entities::PermissionState::Deny)
    );
}

#[tokio::test]
async fn create_custom_role_rejects_duplicate_name() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let viewer = w.roles_by_key["viewer"];
    assert!(roles(&w).create_custom(&ctx, "Admin", viewer).await.is_err());
}

// ── API keys ──────────────────────────────────────────────────
#[tokio::test]
async fn create_key_returns_token_once_and_stores_hash_only() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let created = keys(&w).create(&ctx, "Booking sync", vec![Scope::UsersView, Scope::AuditView]).await.unwrap();
    assert!(created.token.starts_with("ms_live_token"));
    // stored key carries a hash, not the plaintext
    assert_ne!(created.key.token_hash, created.token);
    assert_eq!(created.key.token_hash, format!("h::{}", created.token));
    assert_eq!(w.last_audit_action().as_deref(), Some("key.create"));
}

#[tokio::test]
async fn create_key_requires_a_scope() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    assert!(keys(&w).create(&ctx, "Empty", vec![]).await.is_err());
}

#[tokio::test]
async fn revoke_key_sets_status_and_audits() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let created = keys(&w).create(&ctx, "Temp", vec![Scope::UsersView]).await.unwrap();
    keys(&w).revoke(&ctx, created.key.id).await.unwrap();
    let listed = keys(&w).list(&ctx).await.unwrap();
    let found = listed.iter().find(|k| k.id == created.key.id).unwrap();
    assert_eq!(found.status, core_domain::entities::ApiKeyStatus::Revoked);
    assert_eq!(w.last_audit_action().as_deref(), Some("key.revoke"));
}

// ── Org ───────────────────────────────────────────────────────
#[tokio::test]
async fn update_org_settings_audits() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    let patch = OrgSettingsPatch { name: Some("MadeSpace HQ".into()), ..Default::default() };
    let updated = org(&w).update(&ctx, patch).await.unwrap();
    assert_eq!(updated.name, "MadeSpace HQ");
    assert_eq!(w.last_audit_action().as_deref(), Some("org.settings.update"));
}

#[tokio::test]
async fn ownership_transfer_initiate_then_accept() {
    let w = World::new();
    let owner_ctx = w.ctx_for(w.owner_id).await;
    org(&w).transfer_ownership(&owner_ctx, w.admin_id).await.unwrap();
    // admin accepts
    let admin_ctx = w.ctx_for(w.admin_id).await;
    org(&w).accept_ownership(&admin_ctx).await.unwrap();

    // roles swapped: admin is now owner, old owner is admin
    let admin_now = w.users_repo();
    let new_owner = admin_now.find_by_id(w.admin_id).await.unwrap().unwrap();
    let old_owner = admin_now.find_by_id(w.owner_id).await.unwrap().unwrap();
    assert_eq!(new_owner.role_id, w.roles_by_key["owner"]);
    assert_eq!(old_owner.role_id, w.roles_by_key["admin"]);
}

#[tokio::test]
async fn member_cannot_delete_org() {
    let w = World::new();
    let ctx = w.ctx_for(w.member_id).await;
    let err = org(&w).delete_org(&ctx).await.unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::Forbidden(_))));
}

// ── Audit ─────────────────────────────────────────────────────
#[tokio::test]
async fn audit_search_and_export() {
    let w = World::new();
    let ctx = w.ctx_for(w.owner_id).await;
    // generate a couple of events
    users(&w).invite(&ctx, "z@madespace.co", "member", None).await.unwrap();
    let ctx = w.ctx_for(w.owner_id).await;
    let found = audit(&w).search(&ctx, AuditQuery::default()).await.unwrap();
    assert!(!found.is_empty());

    let csv = audit(&w).export_csv(&ctx, AuditQuery::default()).await.unwrap();
    assert!(csv.starts_with("timestamp,actor,role,action,category"));
    assert!(csv.contains("user.invite"));
}

#[tokio::test]
async fn member_cannot_export_audit() {
    let w = World::new();
    let ctx = w.ctx_for(w.member_id).await;
    assert!(audit(&w).export_csv(&ctx, AuditQuery::default()).await.is_err());
}
