//! Audit-log use-cases: search/filter, and CSV export.

use crate::ctx::ActorContext;
use crate::error::AppError;
use core_domain::entities::AuditEvent;
use core_domain::ports::{AuditQuery, AuditRepo};
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;

#[derive(Clone)]
pub struct AuditService {
    audit: Arc<dyn AuditRepo>,
}

impl AuditService {
    pub fn new(audit: Arc<dyn AuditRepo>) -> Self {
        Self { audit }
    }

    pub async fn search(
        &self,
        ctx: &ActorContext,
        mut query: AuditQuery,
    ) -> Result<Vec<AuditEvent>, AppError> {
        ctx.require("audit.view")?;
        if query.limit <= 0 {
            query.limit = 200;
        }
        Ok(self.audit.search(ctx.actor.org_id, &query).await?)
    }

    /// Export matching events as CSV. Requires the export permission.
    pub async fn export_csv(
        &self,
        ctx: &ActorContext,
        mut query: AuditQuery,
    ) -> Result<String, AppError> {
        ctx.require("audit.export")?;
        query.limit = 100_000;
        query.offset = 0;
        let events = self.audit.search(ctx.actor.org_id, &query).await?;

        let mut out = String::from("timestamp,actor,role,action,category,target,before,after,ip\n");
        for e in &events {
            let ts = e.ts.format(&Rfc3339).unwrap_or_default();
            let row = [
                ts,
                e.actor_name.clone(),
                e.actor_role_key.clone(),
                e.action.clone(),
                e.category.as_str().to_string(),
                e.target.clone().unwrap_or_default(),
                e.before.clone().unwrap_or_default(),
                e.after.clone().unwrap_or_default(),
                e.ip.clone().unwrap_or_default(),
            ];
            out.push_str(
                &row.iter()
                    .map(|f| csv_field(f))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            out.push('\n');
        }
        Ok(out)
    }
}

/// Quote a CSV field if it contains a comma, quote, or newline; double inner quotes.
fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_field_quotes_when_needed() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
