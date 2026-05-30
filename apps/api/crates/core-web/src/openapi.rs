//! The OpenAPI document. `ApiDoc::openapi()` aggregates every `#[utoipa::path]`
//! handler and `ToSchema` DTO into a spec served at `/api-docs/openapi.json` and
//! rendered by Swagger UI at `/swagger-ui`.

use crate::dto::{
    AcceptInviteReq, ActionDto, ApiKeyDto, AuditActorDto, AuditDto, BrandingDto, CellDto,
    CellResultDto, ChangeRoleReq, CreateKeyReq, CreateRoleReq, CreateUserReq, CreatedKeyDto,
    CycleCellReq, ErrorBody, ForgotPasswordReq, GroupDto, InviteReq, LoginReq, MatrixDto, MeDto,
    MfaDto, OrgDto, PasswordPolicyDto, ResetPasswordReq, RoleDto, SetStatusReq, SsoDto,
    TransferReq, UpdateOrgReq, UserDto,
};
use crate::routes;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

/// Declares the cookie-based session scheme referenced by `security(("session" = []))`.
struct SessionCookie;
impl Modify for SessionCookie {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("core_sid"))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Core — MadeSpace Admin API",
        description = "Identity & RBAC admin: users, roles & the permission matrix, org settings, audit log, and service-account API keys. Most endpoints require the `core_sid` session cookie set by `POST /api/auth/login`.",
        version = "0.1.0",
    ),
    paths(
        routes::health,
        routes::auth::login,
        routes::auth::accept_invite,
        routes::auth::forgot_password,
        routes::auth::reset_password,
        routes::auth::logout,
        routes::auth::me,
        routes::users::list,
        routes::users::create,
        routes::users::invite,
        routes::users::change_role,
        routes::users::set_status,
        routes::roles::list,
        routes::roles::create,
        routes::roles::matrix,
        routes::roles::cycle_cell,
        routes::org::get_org,
        routes::org::update,
        routes::org::transfer,
        routes::org::accept,
        routes::org::delete,
        routes::audit::list,
        routes::audit::export,
        routes::keys::list,
        routes::keys::create,
        routes::keys::revoke,
    ),
    components(schemas(
        ErrorBody,
        LoginReq, AcceptInviteReq, ForgotPasswordReq, ResetPasswordReq,
        InviteReq, CreateUserReq, ChangeRoleReq, SetStatusReq,
        CreateRoleReq, CycleCellReq, CreateKeyReq, TransferReq, UpdateOrgReq,
        MfaDto, PasswordPolicyDto, SsoDto,
        UserDto, RoleDto, ActionDto, GroupDto, CellDto, MatrixDto, CellResultDto,
        AuditActorDto, AuditDto, ApiKeyDto, CreatedKeyDto, OrgDto, BrandingDto, MeDto,
    )),
    modifiers(&SessionCookie),
    tags(
        (name = "auth", description = "Sign in/out, invitations, and password reset"),
        (name = "users", description = "User identity: list, create, invite, role & status"),
        (name = "roles", description = "Roles and the permission matrix"),
        (name = "org", description = "Organization settings, ownership transfer, deletion"),
        (name = "audit", description = "Append-only audit log: search and CSV export"),
        (name = "keys", description = "Service-account API keys"),
        (name = "meta", description = "Health and operational endpoints"),
    ),
)]
pub struct ApiDoc;
