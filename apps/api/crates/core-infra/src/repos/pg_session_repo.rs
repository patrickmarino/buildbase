use super::mappers::session_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::Session;
use core_domain::ids::{SessionId, UserId};
use core_domain::ports::{RepoResult, SessionRepo};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgSessionRepo {
    pool: PgPool,
}

impl PgSessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepo for PgSessionRepo {
    async fn create(&self, session: &Session) -> RepoResult<()> {
        sqlx::query(
            "insert into sessions (id, user_id, created_at, expires_at) values ($1,$2,$3,$4)",
        )
        .bind(session.id.as_str())
        .bind(session.user_id.as_uuid())
        .bind(session.created_at)
        .bind(session.expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn get(&self, id: &SessionId) -> RepoResult<Option<Session>> {
        let row = sqlx::query(
            "select id, user_id, created_at, expires_at from sessions where id = $1",
        )
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        row.as_ref().map(session_from_row).transpose()
    }

    async fn delete(&self, id: &SessionId) -> RepoResult<()> {
        sqlx::query("delete from sessions where id = $1")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }

    async fn delete_all_for_user(&self, user: UserId) -> RepoResult<()> {
        sqlx::query("delete from sessions where user_id = $1")
            .bind(user.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }
}
