//! Integration tests for the Postgres repositories. Each test runs against an
//! isolated database created by `#[sqlx::test]` (migrations applied
//! automatically), so they are hermetic. Requires `DATABASE_URL` to point at a
//! Postgres server with create-database privileges.

use core_domain::entities::email::Email;
use core_domain::entities::{
    ApiKey, ApiKeyStatus, AuditEvent, PermissionCategory, PermissionState, Scope, Session, User,
    UserStatus,
};
use core_domain::ids::*;
use core_domain::ports::*;
use core_infra::seed::{ensure_seeded, sole_org_id, SeedConfig};
use core_infra::*;
use sqlx::PgPool;
use time::OffsetDateTime;

fn cfg() -> SeedConfig {
    SeedConfig {
        org_name: "MadeSpace Studio".into(),
        org_domain: Some("madespace.co".into()),
        owner_email: "elena@madespace.co".into(),
        owner_name: "Elena Marchetti".into(),
        owner_password: "owner-password-123!".into(),
    }
}

async fn seed(pool: &PgPool) -> OrgId {
    let hasher = Argon2Hasher;
    ensure_seeded(pool, &hasher, &cfg()).await.unwrap();
    sole_org_id(pool).await.unwrap().unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn seed_is_idempotent_and_creates_roles_matrix_owner(pool: PgPool) {
    let hasher = Argon2Hasher;
    assert!(
        ensure_seeded(&pool, &hasher, &cfg()).await.unwrap(),
        "first seed runs"
    );
    assert!(
        !ensure_seeded(&pool, &hasher, &cfg()).await.unwrap(),
        "second seed no-ops"
    );

    let org = sole_org_id(&pool).await.unwrap().unwrap();
    let roles = PgRoleRepo::new(pool.clone()).list_all(org).await.unwrap();
    assert_eq!(roles.len(), 7, "seven default roles");

    let matrix = PgPermissionRepo::new(pool.clone())
        .load_matrix(org)
        .await
        .unwrap();
    assert_eq!(matrix.roles.len(), 5, "five matrix columns");
    let owner = roles.iter().find(|r| r.key == "owner").unwrap();
    let admin = roles.iter().find(|r| r.key == "admin").unwrap();
    assert_eq!(
        matrix.raw_state("users.invite", owner.id),
        Some(PermissionState::Allow)
    );
    assert_eq!(
        matrix.raw_state("org.delete", admin.id),
        Some(PermissionState::Deny)
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_password_verifies_against_seeded_hash(pool: PgPool) {
    let org = seed(&pool).await;
    let users = PgUserRepo::new(pool.clone());
    let owner = users
        .find_by_email(org, &Email::parse("elena@madespace.co").unwrap())
        .await
        .unwrap()
        .unwrap();
    let hasher = Argon2Hasher;
    let hash = owner.password_hash.unwrap();
    assert!(hasher.verify("owner-password-123!", &hash).unwrap());
    assert!(!hasher.verify("wrong", &hash).unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn user_insert_update_and_admin_count(pool: PgPool) {
    let org = seed(&pool).await;
    let roles = PgRoleRepo::new(pool.clone());
    let users = PgUserRepo::new(pool.clone());
    let admin_role = roles.find_by_key(org, "admin").await.unwrap().unwrap();
    let member_role = roles.find_by_key(org, "member").await.unwrap().unwrap();

    // only the owner so far
    assert_eq!(users.count_active_admins(org).await.unwrap(), 1);

    let now = OffsetDateTime::now_utc();
    let u = User {
        id: UserId::new(),
        org_id: org,
        email: Email::parse("tomas@madespace.co").unwrap(),
        name: "Tomas".into(),
        role_id: admin_role.id,
        status: UserStatus::Active,
        scope: Some("Studio".into()),
        password_hash: None,
        created_at: now,
        last_active_at: None,
    };
    users.insert(&u).await.unwrap();
    assert_eq!(users.count_active_admins(org).await.unwrap(), 2);

    // demote to member → back to 1 admin
    let mut u2 = u.clone();
    u2.role_id = member_role.id;
    users.update(&u2).await.unwrap();
    assert_eq!(users.count_active_admins(org).await.unwrap(), 1);

    // duplicate email is rejected
    let dup = User {
        id: UserId::new(),
        ..u.clone()
    };
    assert!(matches!(
        users.insert(&dup).await,
        Err(RepoError::UniqueViolation(_))
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_filters(pool: PgPool) {
    let org = seed(&pool).await;
    let users = PgUserRepo::new(pool.clone());
    let found = users
        .list(
            org,
            &UserFilter {
                query: Some("elena".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(found.len(), 1);
    let none = users
        .list(
            org,
            &UserFilter {
                status: Some(UserStatus::Deactivated),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(none.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_create_get_and_revoke_all(pool: PgPool) {
    let org = seed(&pool).await;
    let owner = PgUserRepo::new(pool.clone())
        .find_by_email(org, &Email::parse("elena@madespace.co").unwrap())
        .await
        .unwrap()
        .unwrap();
    let sessions = PgSessionRepo::new(pool.clone());
    let now = OffsetDateTime::now_utc();
    let s = Session {
        id: SessionId::new("sid-abc"),
        user_id: owner.id,
        created_at: now,
        expires_at: now + time::Duration::days(1),
    };
    sessions.create(&s).await.unwrap();
    assert!(sessions.get(&s.id).await.unwrap().is_some());

    sessions.delete_all_for_user(owner.id).await.unwrap();
    assert!(sessions.get(&s.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn permission_cell_upsert_roundtrip(pool: PgPool) {
    let org = seed(&pool).await;
    let perms = PgPermissionRepo::new(pool.clone());
    let admin = PgRoleRepo::new(pool.clone())
        .find_by_key(org, "admin")
        .await
        .unwrap()
        .unwrap();

    perms
        .set_cell(org, "users.invite", admin.id, PermissionState::Scope)
        .await
        .unwrap();
    let matrix = perms.load_matrix(org).await.unwrap();
    assert_eq!(
        matrix.raw_state("users.invite", admin.id),
        Some(PermissionState::Scope)
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn audit_append_and_search(pool: PgPool) {
    let org = seed(&pool).await;
    let owner = PgUserRepo::new(pool.clone())
        .find_by_email(org, &Email::parse("elena@madespace.co").unwrap())
        .await
        .unwrap()
        .unwrap();
    let audit = PgAuditRepo::new(pool.clone());
    let ev = AuditEvent {
        id: AuditEventId::new(),
        org_id: org,
        ts: OffsetDateTime::now_utc(),
        actor_id: owner.id,
        actor_name: "Elena Marchetti".into(),
        actor_role_key: "owner".into(),
        action: "permission.edit".into(),
        category: PermissionCategory::Roles,
        target: Some("Admin · Invite user".into()),
        before: Some("Allowed".into()),
        after: Some("Scoped".into()),
        ip: Some("203.0.113.7".into()),
    };
    audit.append(&ev).await.unwrap();

    let all = audit.search(org, &AuditQuery::default()).await.unwrap();
    assert_eq!(all.len(), 1);
    // category filter
    let roles_only = audit
        .search(
            org,
            &AuditQuery {
                category: Some(PermissionCategory::Roles),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(roles_only.len(), 1);
    let users_only = audit
        .search(
            org,
            &AuditQuery {
                category: Some(PermissionCategory::Users),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(users_only.is_empty());
    // text search
    let hit = audit
        .search(
            org,
            &AuditQuery {
                query: Some("invite".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(hit.len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn api_keys_insert_list_revoke(pool: PgPool) {
    let org = seed(&pool).await;
    let keys = PgApiKeyRepo::new(pool.clone());
    let now = OffsetDateTime::now_utc();
    let key = ApiKey {
        id: ApiKeyId::new(),
        org_id: org,
        name: "Booking sync".into(),
        prefix: "ms_live_a91f".into(),
        token_hash: "deadbeef".into(),
        scopes: vec![Scope::UsersView, Scope::AuditView],
        status: ApiKeyStatus::Active,
        created_at: now,
        last_used_at: None,
    };
    keys.insert(&key).await.unwrap();

    let listed = keys.list(org).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].scopes, vec![Scope::UsersView, Scope::AuditView]);

    keys.set_status(key.id, ApiKeyStatus::Revoked)
        .await
        .unwrap();
    let after = keys.find_by_id(key.id).await.unwrap().unwrap();
    assert_eq!(after.status, ApiKeyStatus::Revoked);
}

#[sqlx::test(migrations = "../../migrations")]
async fn ownership_transfer_swaps_roles(pool: PgPool) {
    let org = seed(&pool).await;
    let users = PgUserRepo::new(pool.clone());
    let roles = PgRoleRepo::new(pool.clone());
    let orgs = PgOrgRepo::new(pool.clone());
    let owner_role = roles.find_by_key(org, "owner").await.unwrap().unwrap();
    let admin_role = roles.find_by_key(org, "admin").await.unwrap().unwrap();

    let owner = users
        .find_by_email(org, &Email::parse("elena@madespace.co").unwrap())
        .await
        .unwrap()
        .unwrap();

    // add an admin to receive ownership
    let now = OffsetDateTime::now_utc();
    let admin = User {
        id: UserId::new(),
        org_id: org,
        email: Email::parse("tomas@madespace.co").unwrap(),
        name: "Tomas".into(),
        role_id: admin_role.id,
        status: UserStatus::Active,
        scope: None,
        password_hash: None,
        created_at: now,
        last_active_at: None,
    };
    users.insert(&admin).await.unwrap();

    orgs.complete_ownership_transfer(org, admin.id, owner.id, owner_role.id, admin_role.id)
        .await
        .unwrap();

    let new_owner = users.find_by_id(admin.id).await.unwrap().unwrap();
    let old_owner = users.find_by_id(owner.id).await.unwrap().unwrap();
    assert_eq!(new_owner.role_id, owner_role.id);
    assert_eq!(old_owner.role_id, admin_role.id);
    let org_row = orgs.get(org).await.unwrap().unwrap();
    assert_eq!(org_row.owner_id, admin.id);
}
