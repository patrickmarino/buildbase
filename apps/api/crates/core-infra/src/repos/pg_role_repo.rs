use super::mappers::role_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::Role;
use core_domain::ids::{OrgId, RoleId};
use core_domain::ports::{RepoResult, RoleRepo};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgRoleRepo {
    pool: PgPool,
}

impl PgRoleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const COLS: &str = "id, org_id, key, name, builtin, is_column, base_role_id, rank";

#[async_trait]
impl RoleRepo for PgRoleRepo {
    async fn find_by_id(&self, id: RoleId) -> RepoResult<Option<Role>> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!("select {COLS} from roles where id = $1")))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;
        row.as_ref().map(role_from_row).transpose()
    }

    async fn find_by_key(&self, org: OrgId, key: &str) -> RepoResult<Option<Role>> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "select {COLS} from roles where org_id = $1 and key = $2"
        )))
        .bind(org.as_uuid())
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        row.as_ref().map(role_from_row).transpose()
    }

    async fn list_all(&self, org: OrgId) -> RepoResult<Vec<Role>> {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "select {COLS} from roles where org_id = $1 order by rank desc, name asc"
        )))
        .bind(org.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        rows.iter().map(role_from_row).collect()
    }

    async fn insert(&self, role: &Role) -> RepoResult<()> {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into roles ({COLS}) values ($1,$2,$3,$4,$5,$6,$7,$8)"
        )))
        .bind(role.id.as_uuid())
        .bind(role.org_id.as_uuid())
        .bind(&role.key)
        .bind(&role.name)
        .bind(role.builtin.map(|b| b.key()))
        .bind(role.is_column)
        .bind(role.base_role_id.map(|r| r.as_uuid()))
        .bind(role.rank)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }
}
