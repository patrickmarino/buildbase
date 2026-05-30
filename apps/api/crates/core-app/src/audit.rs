//! The shared auditing helper. Every mutating use-case records an event through
//! this, so the audit log is the single, uniform record of sensitive actions.

use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::{AuditEvent, PermissionCategory};
use core_domain::ids::AuditEventId;
use core_domain::ports::{AuditRepo, Clock};
use std::sync::Arc;

#[derive(Clone)]
pub struct Auditor {
    audit: Arc<dyn AuditRepo>,
    clock: Arc<dyn Clock>,
}

impl Auditor {
    pub fn new(audit: Arc<dyn AuditRepo>, clock: Arc<dyn Clock>) -> Self {
        Self { audit, clock }
    }

    /// Append an audit event derived from the actor context.
    pub async fn record(
        &self,
        ctx: &ActorContext,
        action: &str,
        category: PermissionCategory,
        target: Option<String>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<(), AppError> {
        let event = AuditEvent {
            id: AuditEventId::new(),
            org_id: ctx.actor.org_id,
            ts: self.clock.now(),
            actor_id: ctx.actor.id,
            actor_name: ctx.actor.name.clone(),
            actor_role_key: ctx.actor_role.key.clone(),
            action: action.to_string(),
            category,
            target,
            before,
            after,
            ip: ctx.ip.clone(),
        };
        self.audit.append(&event).await?;
        Ok(())
    }
}
