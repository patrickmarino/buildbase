use super::mappers::audit_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::AuditEvent;
use core_domain::ids::OrgId;
use core_domain::ports::{AuditQuery, AuditRepo, RepoResult};
use sqlx::{PgPool, QueryBuilder};

#[derive(Clone)]
pub struct PgAuditRepo {
    pool: PgPool,
}

impl PgAuditRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const COLS: &str = "id, org_id, ts, actor_id, actor_name, actor_role_key, action, category, \
    target, before_val, after_val, ip";

#[async_trait]
impl AuditRepo for PgAuditRepo {
    async fn append(&self, e: &AuditEvent) -> RepoResult<()> {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into audit_events ({COLS}) values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
        )))
        .bind(e.id.as_uuid())
        .bind(e.org_id.as_uuid())
        .bind(e.ts)
        .bind(e.actor_id.as_uuid())
        .bind(&e.actor_name)
        .bind(&e.actor_role_key)
        .bind(&e.action)
        .bind(e.category.as_str())
        .bind(&e.target)
        .bind(&e.before)
        .bind(&e.after)
        .bind(&e.ip)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn search(&self, org: OrgId, query: &AuditQuery) -> RepoResult<Vec<AuditEvent>> {
        let mut qb = QueryBuilder::new(format!("select {COLS} from audit_events where org_id = "));
        qb.push_bind(org.as_uuid());
        if let Some(cat) = query.category {
            qb.push(" and category = ").push_bind(cat.as_str());
        }
        if let Some(q) = &query.query {
            let pat = format!("%{}%", q.to_lowercase());
            qb.push(" and (lower(actor_name) like ")
                .push_bind(pat.clone())
                .push(" or lower(action) like ")
                .push_bind(pat.clone())
                .push(" or lower(coalesce(target,'')) like ")
                .push_bind(pat)
                .push(")");
        }
        qb.push(" order by ts desc");
        let limit = if query.limit > 0 { query.limit } else { 200 };
        qb.push(" limit ").push_bind(limit);
        qb.push(" offset ").push_bind(query.offset.max(0));

        let rows = qb.build().fetch_all(&self.pool).await.map_err(map_sqlx)?;
        rows.iter().map(audit_from_row).collect()
    }
}
