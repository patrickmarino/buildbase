//! Append-only audit events — actor, action, target, before/after, time, IP.

use crate::entities::permission::PermissionCategory;
use crate::ids::{AuditEventId, OrgId, UserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub org_id: OrgId,
    pub ts: OffsetDateTime,
    pub actor_id: UserId,
    /// Snapshot of the actor's role key at the time of the action.
    pub actor_name: String,
    pub actor_role_key: String,
    /// Dotted action name, e.g. `permission.edit`, `user.invite`.
    pub action: String,
    pub category: PermissionCategory,
    pub target: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ip: Option<String>,
}
