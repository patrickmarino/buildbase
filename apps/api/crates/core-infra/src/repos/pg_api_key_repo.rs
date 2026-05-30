use super::mappers::api_key_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::{ApiKey, ApiKeyStatus};
use core_domain::ids::{ApiKeyId, OrgId};
use core_domain::ports::{ApiKeyRepo, RepoResult};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgApiKeyRepo {
    pool: PgPool,
}

impl PgApiKeyRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const COLS: &str =
    "id, org_id, name, prefix, token_hash, scopes, status, created_at, last_used_at";

#[async_trait]
impl ApiKeyRepo for PgApiKeyRepo {
    async fn list(&self, org: OrgId) -> RepoResult<Vec<ApiKey>> {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "select {COLS} from api_keys where org_id = $1 order by created_at desc"
        )))
        .bind(org.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        rows.iter().map(api_key_from_row).collect()
    }

    async fn find_by_id(&self, id: ApiKeyId) -> RepoResult<Option<ApiKey>> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!("select {COLS} from api_keys where id = $1")))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;
        row.as_ref().map(api_key_from_row).transpose()
    }

    async fn insert(&self, key: &ApiKey) -> RepoResult<()> {
        let scopes = serde_json::Value::Array(
            key.scopes
                .iter()
                .map(|s| serde_json::Value::String(s.as_str().to_string()))
                .collect(),
        );
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into api_keys ({COLS}) values ($1,$2,$3,$4,$5,$6,$7,$8,$9)"
        )))
        .bind(key.id.as_uuid())
        .bind(key.org_id.as_uuid())
        .bind(&key.name)
        .bind(&key.prefix)
        .bind(&key.token_hash)
        .bind(scopes)
        .bind(key.status.as_str())
        .bind(key.created_at)
        .bind(key.last_used_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn set_status(&self, id: ApiKeyId, status: ApiKeyStatus) -> RepoResult<()> {
        sqlx::query("update api_keys set status = $2 where id = $1")
            .bind(id.as_uuid())
            .bind(status.as_str())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }
}
