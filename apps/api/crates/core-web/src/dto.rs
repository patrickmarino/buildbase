//! Request and response DTOs, and their mapping to/from domain types. DTOs are
//! camelCase JSON, with enums as the prototype's lowercase strings. They live
//! only here — the domain never derives web-facing shapes.

use core_app::ActorContext;
use core_domain::entities::organization::{
    MfaConfig, MfaEnforce, MfaMethod, Organization, PasswordPolicy, SsoConfig, SsoProvider,
};
use core_domain::entities::role::BuiltinRole;
use core_domain::entities::{ApiKey, AuditEvent, MatrixCell, PermissionMatrix, Role, User};
use core_domain::ids::RoleId;
use core_domain::services::authz::{can, AuthzInput};
use core_domain::services::matrix_rules;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

// ── time helpers ──────────────────────────────────────────────
fn iso(t: OffsetDateTime) -> String {
    t.format(&Rfc3339).unwrap_or_default()
}
fn iso_opt(t: Option<OffsetDateTime>) -> Option<String> {
    t.map(iso)
}

// ════════════════════════════════════════════════════════════
// Requests
// ════════════════════════════════════════════════════════════
#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteReq {
    pub email: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserReq {
    #[serde(default)]
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub scope: Option<String>,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRoleReq {
    pub role_id: uuid::Uuid,
}

#[derive(Deserialize)]
pub struct SetStatusReq {
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleReq {
    pub name: String,
    pub base_role_id: uuid::Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleCellReq {
    pub action_key: String,
    pub role_id: uuid::Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyReq {
    pub name: String,
    pub scopes: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferReq {
    pub target_user_id: uuid::Uuid,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrgReq {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub accent_color: Option<String>,
    pub mfa: Option<MfaDto>,
    pub password_policy: Option<PasswordPolicyDto>,
    pub sso: Option<SsoDto>,
}

// ════════════════════════════════════════════════════════════
// Shared sub-DTOs (round-trip)
// ════════════════════════════════════════════════════════════
#[derive(Serialize, Deserialize, Clone)]
pub struct MfaDto {
    pub enabled: bool,
    pub method: String,
    pub enforce: String,
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PasswordPolicyDto {
    pub min_length: u8,
    pub require_number: bool,
    pub require_symbol: bool,
    pub rotation_days: Option<u16>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct SsoDto {
    pub enabled: bool,
    pub provider: String,
    pub url: Option<String>,
}

impl MfaDto {
    pub fn into_domain(self) -> MfaConfig {
        MfaConfig {
            enabled: self.enabled,
            method: match self.method.as_str() {
                "sms" => MfaMethod::Sms,
                "webauthn" => MfaMethod::Webauthn,
                _ => MfaMethod::Totp,
            },
            enforce: match self.enforce.as_str() {
                "all" => MfaEnforce::All,
                "none" => MfaEnforce::None,
                _ => MfaEnforce::Admins,
            },
        }
    }
}
impl PasswordPolicyDto {
    pub fn into_domain(self) -> PasswordPolicy {
        PasswordPolicy {
            min_length: self.min_length,
            require_number: self.require_number,
            require_symbol: self.require_symbol,
            rotation_days: self.rotation_days,
        }
    }
}
impl SsoDto {
    pub fn into_domain(self) -> SsoConfig {
        SsoConfig {
            enabled: self.enabled,
            provider: if self.provider == "oidc" {
                SsoProvider::Oidc
            } else {
                SsoProvider::Saml
            },
            url: self.url,
        }
    }
}

// ════════════════════════════════════════════════════════════
// Responses
// ════════════════════════════════════════════════════════════
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role_id: String,
    pub role_key: String,
    pub role_name: String,
    pub status: String,
    pub scope: Option<String>,
    pub last_active: Option<String>,
    pub created_at: String,
}

pub fn user_dto(u: &User, roles: &HashMap<RoleId, Role>) -> UserDto {
    let role = roles.get(&u.role_id);
    UserDto {
        id: u.id.to_string(),
        name: u.name.clone(),
        email: u.email.to_string(),
        role_id: u.role_id.to_string(),
        role_key: role.map(|r| r.key.clone()).unwrap_or_default(),
        role_name: role.map(|r| r.name.clone()).unwrap_or_default(),
        status: u.status.as_str().to_string(),
        scope: u.scope.clone(),
        last_active: iso_opt(u.last_active_at),
        created_at: iso(u.created_at),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDto {
    pub id: String,
    pub key: String,
    pub name: String,
    pub is_column: bool,
    pub custom: bool,
    pub protected: bool,
    pub inherits: String,
    pub rank: i16,
}

pub fn role_dto(r: &Role, roles: &HashMap<RoleId, Role>) -> RoleDto {
    RoleDto {
        id: r.id.to_string(),
        key: r.key.clone(),
        name: r.name.clone(),
        is_column: r.is_column,
        custom: r.is_custom(),
        protected: r.is_owner(),
        inherits: inherits_label(r, roles),
        rank: r.rank,
    }
}

fn inherits_label(r: &Role, roles: &HashMap<RoleId, Role>) -> String {
    if let Some(base) = r.base_role_id.and_then(|id| roles.get(&id)) {
        return base.name.clone();
    }
    match r.builtin {
        Some(BuiltinRole::Owner) => "Admin",
        Some(BuiltinRole::Admin) => "Manager",
        Some(BuiltinRole::Manager) => "Member",
        Some(BuiltinRole::Member) => "Viewer",
        _ => "—",
    }
    .to_string()
}

#[derive(Serialize)]
pub struct ActionDto {
    pub key: String,
    pub label: String,
}
#[derive(Serialize)]
pub struct GroupDto {
    pub category: String,
    pub label: String,
    pub actions: Vec<ActionDto>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellDto {
    pub action_key: String,
    pub role_id: String,
    pub state: String,
    pub locked: bool,
}
#[derive(Serialize)]
pub struct MatrixDto {
    pub groups: Vec<GroupDto>,
    pub columns: Vec<RoleDto>,
    pub cells: Vec<CellDto>,
}

pub fn matrix_dto(m: &PermissionMatrix, roles: &HashMap<RoleId, Role>) -> MatrixDto {
    let groups = m
        .groups
        .iter()
        .map(|g| GroupDto {
            category: g.category.as_str().to_string(),
            label: g.category.label().to_string(),
            actions: g
                .actions
                .iter()
                .map(|a| ActionDto {
                    key: a.key.clone(),
                    label: a.label.clone(),
                })
                .collect(),
        })
        .collect();

    // Column roles, ordered most → least powerful (matrix order).
    let columns = m
        .roles
        .iter()
        .filter_map(|(id, _)| roles.get(id))
        .map(|r| role_dto(r, roles))
        .collect();

    let cells = m
        .cells
        .iter()
        .map(|c: &MatrixCell| {
            let role_key = roles.get(&c.role_id).map(|r| r.key.as_str()).unwrap_or("");
            CellDto {
                action_key: c.action_key.clone(),
                role_id: c.role_id.to_string(),
                state: c.state.as_str().to_string(),
                locked: matrix_rules::is_locked(&c.action_key, role_key),
            }
        })
        .collect();

    MatrixDto {
        groups,
        columns,
        cells,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellResultDto {
    pub action_key: String,
    pub role_id: String,
    pub state: String,
}
pub fn cell_result_dto(c: &MatrixCell) -> CellResultDto {
    CellResultDto {
        action_key: c.action_key.clone(),
        role_id: c.role_id.to_string(),
        state: c.state.as_str().to_string(),
    }
}

#[derive(Serialize)]
pub struct AuditActorDto {
    pub name: String,
    pub role: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditDto {
    pub id: String,
    pub ts: String,
    pub actor: AuditActorDto,
    pub action: String,
    pub category: String,
    pub target: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ip: Option<String>,
}
pub fn audit_dto(e: &AuditEvent) -> AuditDto {
    AuditDto {
        id: e.id.to_string(),
        ts: iso(e.ts),
        actor: AuditActorDto {
            name: e.actor_name.clone(),
            role: e.actor_role_key.clone(),
        },
        action: e.action.clone(),
        category: e.category.as_str().to_string(),
        target: e.target.clone(),
        before: e.before.clone(),
        after: e.after.clone(),
        ip: e.ip.clone(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyDto {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub status: String,
    pub created: String,
    pub last_used: Option<String>,
}
pub fn api_key_dto(k: &ApiKey) -> ApiKeyDto {
    ApiKeyDto {
        id: k.id.to_string(),
        name: k.name.clone(),
        prefix: k.prefix.clone(),
        scopes: k.scopes.iter().map(|s| s.as_str().to_string()).collect(),
        status: k.status.as_str().to_string(),
        created: iso(k.created_at),
        last_used: iso_opt(k.last_used_at),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedKeyDto {
    pub key: ApiKeyDto,
    pub token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgDto {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub branding: BrandingDto,
    pub mfa: MfaDto,
    pub password_policy: PasswordPolicyDto,
    pub sso: SsoDto,
    pub owner_id: String,
    pub pending_owner_id: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandingDto {
    pub accent_color: String,
}
pub fn org_dto(o: &Organization) -> OrgDto {
    OrgDto {
        id: o.id.to_string(),
        name: o.name.clone(),
        domain: o.domain.clone(),
        branding: BrandingDto {
            accent_color: o.branding.accent_color.clone(),
        },
        mfa: MfaDto {
            enabled: o.mfa.enabled,
            method: match o.mfa.method {
                MfaMethod::Totp => "totp",
                MfaMethod::Sms => "sms",
                MfaMethod::Webauthn => "webauthn",
            }
            .into(),
            enforce: match o.mfa.enforce {
                MfaEnforce::All => "all",
                MfaEnforce::Admins => "admins",
                MfaEnforce::None => "none",
            }
            .into(),
        },
        password_policy: PasswordPolicyDto {
            min_length: o.password_policy.min_length,
            require_number: o.password_policy.require_number,
            require_symbol: o.password_policy.require_symbol,
            rotation_days: o.password_policy.rotation_days,
        },
        sso: SsoDto {
            enabled: o.sso.enabled,
            provider: match o.sso.provider {
                SsoProvider::Saml => "saml",
                SsoProvider::Oidc => "oidc",
            }
            .into(),
            url: o.sso.url.clone(),
        },
        owner_id: o.owner_id.to_string(),
        pending_owner_id: o.pending_owner_id.map(|u| u.to_string()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeDto {
    pub user: UserDto,
    pub permissions: Vec<String>,
}

/// The `/auth/me` payload: the actor plus the action keys they may perform
/// (Allow or self-Scope), so the SPA can gate UI without re-deriving rules.
pub fn me_dto(ctx: &ActorContext) -> MeDto {
    let mut roles = HashMap::new();
    roles.insert(ctx.actor_role.id, ctx.actor_role.clone());
    let user = user_dto(&ctx.actor, &roles);

    let permissions = ctx
        .matrix
        .actions()
        .filter(|a| {
            let input = AuthzInput {
                actor_role: ctx.actor_role.id,
                matrix: &ctx.matrix,
                actor_id: ctx.actor.id,
                resource_owner: Some(ctx.actor.id),
            };
            can(&input, &a.key).is_allowed()
        })
        .map(|a| a.key.clone())
        .collect();

    MeDto { user, permissions }
}
