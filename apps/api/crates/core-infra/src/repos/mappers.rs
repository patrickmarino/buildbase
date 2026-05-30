//! Map Postgres rows onto domain entities. Enums are stored as `TEXT`; ids are
//! `uuid`. Decoding failures become `RepoError::Other` via [`decode_err`].

use crate::error::decode_err;
use core_domain::entities::email::Email;
use core_domain::entities::organization::{
    Branding, MfaConfig, MfaEnforce, MfaMethod, PasswordPolicy, SsoConfig, SsoProvider,
};
use core_domain::entities::role::BuiltinRole;
use core_domain::entities::{
    ApiKey, ApiKeyStatus, AuditEvent, Organization, PermissionCategory, PermissionState, Role,
    Scope, Session, User, UserStatus,
};
use core_domain::ids::*;
use core_domain::ports::{RepoError, RepoResult};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

pub fn user_from_row(row: &PgRow) -> RepoResult<User> {
    let status_s: String = row.try_get("status").map_err(into_repo)?;
    let email_s: String = row.try_get("email").map_err(into_repo)?;
    Ok(User {
        id: UserId::from_uuid(row.try_get("id").map_err(into_repo)?),
        org_id: OrgId::from_uuid(row.try_get("org_id").map_err(into_repo)?),
        email: Email::parse(&email_s).map_err(|_| decode_err("email", &email_s))?,
        name: row.try_get("name").map_err(into_repo)?,
        role_id: RoleId::from_uuid(row.try_get("role_id").map_err(into_repo)?),
        status: UserStatus::from_str(&status_s)
            .ok_or_else(|| decode_err("user status", &status_s))?,
        scope: row.try_get("scope").map_err(into_repo)?,
        password_hash: row.try_get("password_hash").map_err(into_repo)?,
        created_at: row.try_get("created_at").map_err(into_repo)?,
        last_active_at: row.try_get("last_active_at").map_err(into_repo)?,
    })
}

pub fn role_from_row(row: &PgRow) -> RepoResult<Role> {
    let builtin_s: Option<String> = row.try_get("builtin").map_err(into_repo)?;
    let builtin = match builtin_s {
        Some(s) => Some(BuiltinRole::from_key(&s).ok_or_else(|| decode_err("builtin role", &s))?),
        None => None,
    };
    let base: Option<Uuid> = row.try_get("base_role_id").map_err(into_repo)?;
    Ok(Role {
        id: RoleId::from_uuid(row.try_get("id").map_err(into_repo)?),
        org_id: OrgId::from_uuid(row.try_get("org_id").map_err(into_repo)?),
        key: row.try_get("key").map_err(into_repo)?,
        name: row.try_get("name").map_err(into_repo)?,
        builtin,
        is_column: row.try_get("is_column").map_err(into_repo)?,
        base_role_id: base.map(RoleId::from_uuid),
        rank: row.try_get("rank").map_err(into_repo)?,
    })
}

pub fn org_from_row(row: &PgRow) -> RepoResult<Organization> {
    let mfa_method_s: String = row.try_get("mfa_method").map_err(into_repo)?;
    let mfa_enforce_s: String = row.try_get("mfa_enforce").map_err(into_repo)?;
    let sso_provider_s: String = row.try_get("sso_provider").map_err(into_repo)?;
    let pw_rotation: Option<i16> = row.try_get("pw_rotation_days").map_err(into_repo)?;
    let pw_min: i16 = row.try_get("pw_min_length").map_err(into_repo)?;
    let owner: Option<Uuid> = row.try_get("owner_id").map_err(into_repo)?;
    let pending: Option<Uuid> = row.try_get("pending_owner_id").map_err(into_repo)?;

    Ok(Organization {
        id: OrgId::from_uuid(row.try_get("id").map_err(into_repo)?),
        name: row.try_get("name").map_err(into_repo)?,
        domain: row.try_get("domain").map_err(into_repo)?,
        owner_id: UserId::from_uuid(owner.unwrap_or_default()),
        pending_owner_id: pending.map(UserId::from_uuid),
        branding: Branding {
            accent_color: row.try_get("branding_accent").map_err(into_repo)?,
        },
        mfa: MfaConfig {
            enabled: row.try_get("mfa_enabled").map_err(into_repo)?,
            method: parse_mfa_method(&mfa_method_s)?,
            enforce: parse_mfa_enforce(&mfa_enforce_s)?,
        },
        password_policy: PasswordPolicy {
            min_length: pw_min.clamp(0, 255) as u8,
            require_number: row.try_get("pw_require_number").map_err(into_repo)?,
            require_symbol: row.try_get("pw_require_symbol").map_err(into_repo)?,
            rotation_days: pw_rotation.map(|d| d.max(0) as u16),
        },
        sso: SsoConfig {
            enabled: row.try_get("sso_enabled").map_err(into_repo)?,
            provider: parse_sso_provider(&sso_provider_s)?,
            url: row.try_get("sso_url").map_err(into_repo)?,
        },
    })
}

pub fn audit_from_row(row: &PgRow) -> RepoResult<AuditEvent> {
    let cat_s: String = row.try_get("category").map_err(into_repo)?;
    Ok(AuditEvent {
        id: AuditEventId::from_uuid(row.try_get("id").map_err(into_repo)?),
        org_id: OrgId::from_uuid(row.try_get("org_id").map_err(into_repo)?),
        ts: row.try_get("ts").map_err(into_repo)?,
        actor_id: UserId::from_uuid(row.try_get("actor_id").map_err(into_repo)?),
        actor_name: row.try_get("actor_name").map_err(into_repo)?,
        actor_role_key: row.try_get("actor_role_key").map_err(into_repo)?,
        action: row.try_get("action").map_err(into_repo)?,
        category: PermissionCategory::from_str(&cat_s)
            .ok_or_else(|| decode_err("category", &cat_s))?,
        target: row.try_get("target").map_err(into_repo)?,
        before: row.try_get("before_val").map_err(into_repo)?,
        after: row.try_get("after_val").map_err(into_repo)?,
        ip: row.try_get("ip").map_err(into_repo)?,
    })
}

pub fn api_key_from_row(row: &PgRow) -> RepoResult<ApiKey> {
    let status_s: String = row.try_get("status").map_err(into_repo)?;
    let scopes_json: serde_json::Value = row.try_get("scopes").map_err(into_repo)?;
    let scopes = scopes_json
        .as_array()
        .ok_or_else(|| decode_err("scopes", "non-array"))?
        .iter()
        .filter_map(|v| v.as_str())
        .filter_map(Scope::from_str)
        .collect();
    Ok(ApiKey {
        id: ApiKeyId::from_uuid(row.try_get("id").map_err(into_repo)?),
        org_id: OrgId::from_uuid(row.try_get("org_id").map_err(into_repo)?),
        name: row.try_get("name").map_err(into_repo)?,
        prefix: row.try_get("prefix").map_err(into_repo)?,
        token_hash: row.try_get("token_hash").map_err(into_repo)?,
        scopes,
        status: ApiKeyStatus::from_str(&status_s)
            .ok_or_else(|| decode_err("key status", &status_s))?,
        created_at: row.try_get("created_at").map_err(into_repo)?,
        last_used_at: row.try_get("last_used_at").map_err(into_repo)?,
    })
}

pub fn session_from_row(row: &PgRow) -> RepoResult<Session> {
    Ok(Session {
        id: SessionId::new(row.try_get::<String, _>("id").map_err(into_repo)?),
        user_id: UserId::from_uuid(row.try_get("user_id").map_err(into_repo)?),
        created_at: row.try_get("created_at").map_err(into_repo)?,
        expires_at: row.try_get("expires_at").map_err(into_repo)?,
    })
}

pub fn cell_state_from_row(row: &PgRow) -> RepoResult<(String, RoleId, PermissionState)> {
    let state_s: String = row.try_get("state").map_err(into_repo)?;
    Ok((
        row.try_get("action_key").map_err(into_repo)?,
        RoleId::from_uuid(row.try_get("role_id").map_err(into_repo)?),
        PermissionState::from_str(&state_s)
            .ok_or_else(|| decode_err("permission state", &state_s))?,
    ))
}

// ── small parsers ─────────────────────────────────────────────
fn parse_mfa_method(s: &str) -> RepoResult<MfaMethod> {
    Ok(match s {
        "totp" => MfaMethod::Totp,
        "sms" => MfaMethod::Sms,
        "webauthn" => MfaMethod::Webauthn,
        _ => return Err(decode_err("mfa method", s)),
    })
}
fn parse_mfa_enforce(s: &str) -> RepoResult<MfaEnforce> {
    Ok(match s {
        "all" => MfaEnforce::All,
        "admins" => MfaEnforce::Admins,
        "none" => MfaEnforce::None,
        _ => return Err(decode_err("mfa enforce", s)),
    })
}
fn parse_sso_provider(s: &str) -> RepoResult<SsoProvider> {
    Ok(match s {
        "saml" => SsoProvider::Saml,
        "oidc" => SsoProvider::Oidc,
        _ => return Err(decode_err("sso provider", s)),
    })
}

// ── serializers (domain enum → db text) ───────────────────────
pub fn mfa_method_str(m: MfaMethod) -> &'static str {
    match m {
        MfaMethod::Totp => "totp",
        MfaMethod::Sms => "sms",
        MfaMethod::Webauthn => "webauthn",
    }
}
pub fn mfa_enforce_str(m: MfaEnforce) -> &'static str {
    match m {
        MfaEnforce::All => "all",
        MfaEnforce::Admins => "admins",
        MfaEnforce::None => "none",
    }
}
pub fn sso_provider_str(p: SsoProvider) -> &'static str {
    match p {
        SsoProvider::Saml => "saml",
        SsoProvider::Oidc => "oidc",
    }
}

fn into_repo(e: sqlx::Error) -> RepoError {
    RepoError::Other(e.to_string())
}
