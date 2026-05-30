//! The organization: profile, branding, and the security foundation
//! (MFA, password policy, SSO) every other module trusts.

use crate::ids::{OrgId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Totp,
    Sms,
    Webauthn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaEnforce {
    All,
    Admins,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub method: MfaMethod,
    pub enforce: MfaEnforce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u8,
    pub require_number: bool,
    pub require_symbol: bool,
    /// `None` = never rotate; otherwise rotate every N days.
    pub rotation_days: Option<u16>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        // Matches the prototype's default org settings.
        Self {
            min_length: 12,
            require_number: true,
            require_symbol: true,
            rotation_days: Some(90),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SsoProvider {
    Saml,
    Oidc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsoConfig {
    pub enabled: bool,
    pub provider: SsoProvider,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branding {
    /// The champagne accent that runs through every surface, e.g. `#D6B982`.
    pub accent_color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrgId,
    pub name: String,
    pub domain: Option<String>,
    pub owner_id: UserId,
    /// Set while an ownership transfer is awaiting acceptance.
    pub pending_owner_id: Option<UserId>,
    pub branding: Branding,
    pub mfa: MfaConfig,
    pub password_policy: PasswordPolicy,
    pub sso: SsoConfig,
}
