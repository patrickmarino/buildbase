//! Idempotent first-boot seed. When the database has no users yet, creates the
//! default organization, the seven roles, the full permission matrix (from the
//! canonical `core_domain::seed` data), and the Owner — with a real Argon2 hash
//! so the frontend can sign in.

use crate::error::map_sqlx;
use core_domain::entities::email::Email;
use core_domain::entities::role::BuiltinRole;
use core_domain::ids::{OrgId, RoleId, UserId};
use core_domain::ports::{PasswordHasher, RepoResult};
use core_domain::seed as dseed;
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

/// Configuration for the seeded org and Owner. Sourced from env in `core-web`.
pub struct SeedConfig {
    pub org_name: String,
    pub org_domain: Option<String>,
    pub owner_email: String,
    pub owner_name: String,
    pub owner_password: String,
}

/// Seed the default dataset if the database is empty. Returns `Ok(true)` when it
/// seeded, `Ok(false)` when users already existed (no-op).
pub async fn ensure_seeded(
    pool: &sqlx::PgPool,
    hasher: &dyn PasswordHasher,
    cfg: &SeedConfig,
) -> RepoResult<bool> {
    let existing: i64 = sqlx::query_scalar("select count(*) from users")
        .fetch_one(pool)
        .await
        .map_err(map_sqlx)?;
    if existing > 0 {
        return Ok(false);
    }

    let email = Email::parse(&cfg.owner_email)
        .map_err(|_| core_domain::ports::RepoError::Other("invalid SEED_OWNER_EMAIL".into()))?;
    let hash = hasher
        .hash(&cfg.owner_password)
        .map_err(|e| core_domain::ports::RepoError::Other(format!("seed hash: {e}")))?;

    let org_id = OrgId::new();
    let owner_id = UserId::new();
    let now = OffsetDateTime::now_utc();

    let mut tx = pool.begin().await.map_err(map_sqlx)?;

    // Organization (owner_id set up-front; no FK on it).
    sqlx::query(
        "insert into organizations (id, name, domain, owner_id, pending_owner_id, branding_accent, \
         mfa_enabled, mfa_method, mfa_enforce, pw_min_length, pw_require_number, pw_require_symbol, \
         pw_rotation_days, sso_enabled, sso_provider, sso_url, created_at) \
         values ($1,$2,$3,$4,null,$5, true,'totp','admins', 12,true,true,90, false,'saml',null, $6)",
    )
    .bind(org_id.as_uuid())
    .bind(&cfg.org_name)
    .bind(&cfg.org_domain)
    .bind(owner_id.as_uuid())
    .bind(dseed::default_branding_accent())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx)?;

    // Roles
    let mut role_ids: HashMap<&'static str, RoleId> = HashMap::new();
    for b in BuiltinRole::ordered() {
        let id = RoleId::new();
        role_ids.insert(b.key(), id);
        sqlx::query(
            "insert into roles (id, org_id, key, name, builtin, is_column, base_role_id, rank) \
             values ($1,$2,$3,$4,$5,$6,null,$7)",
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(b.key())
        .bind(b.display_name())
        .bind(b.key())
        .bind(b.is_column())
        .bind(b.rank())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx)?;
    }

    // Permission cells
    for (action_key, role_key, state) in dseed::default_cells() {
        let Some(role_id) = role_ids.get(role_key) else {
            continue;
        };
        sqlx::query(
            "insert into permission_cells (org_id, role_id, action_key, state) values ($1,$2,$3,$4)",
        )
        .bind(org_id.as_uuid())
        .bind(role_id.as_uuid())
        .bind(action_key)
        .bind(state.as_str())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx)?;
    }

    // Owner user
    let owner_role = role_ids["owner"];
    sqlx::query(
        "insert into users (id, org_id, email, name, role_id, status, scope, password_hash, created_at, last_active_at) \
         values ($1,$2,$3,$4,$5,'active','Studio',$6,$7,$7)",
    )
    .bind(owner_id.as_uuid())
    .bind(org_id.as_uuid())
    .bind(email.as_str())
    .bind(&cfg.owner_name)
    .bind(owner_role.as_uuid())
    .bind(&hash)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(map_sqlx)?;

    tx.commit().await.map_err(map_sqlx)?;
    Ok(true)
}

/// The id of the single seeded organization, if exactly one exists. Used by the
/// login endpoint to resolve which org a sign-in belongs to (single-tenant dev).
pub async fn sole_org_id(pool: &sqlx::PgPool) -> RepoResult<Option<OrgId>> {
    let rows: Vec<Uuid> = sqlx::query_scalar("select id from organizations limit 2")
        .fetch_all(pool)
        .await
        .map_err(map_sqlx)?;
    Ok(if rows.len() == 1 {
        Some(OrgId::from_uuid(rows[0]))
    } else {
        None
    })
}
