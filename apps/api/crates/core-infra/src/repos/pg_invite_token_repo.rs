use super::mappers::invite_token_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::InviteToken;
use core_domain::ids::{InviteTokenId, UserId};
use core_domain::ports::{InviteTokenRepo, RepoResult};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgInviteTokenRepo {
    pool: PgPool,
}

impl PgInviteTokenRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InviteTokenRepo for PgInviteTokenRepo {
    async fn insert(&self, token: &InviteToken) -> RepoResult<()> {
        sqlx::query(
            "insert into invite_tokens (id, user_id, token_hash, created_at, expires_at) \
             values ($1, $2, $3, $4, $5)",
        )
        .bind(token.id.as_uuid())
        .bind(token.user_id.as_uuid())
        .bind(&token.token_hash)
        .bind(token.created_at)
        .bind(token.expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn find_by_hash(&self, token_hash: &str) -> RepoResult<Option<InviteToken>> {
        let row = sqlx::query(
            "select id, user_id, token_hash, created_at, expires_at \
             from invite_tokens where token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        row.as_ref().map(invite_token_from_row).transpose()
    }

    async fn delete(&self, id: InviteTokenId) -> RepoResult<()> {
        sqlx::query("delete from invite_tokens where id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }

    async fn delete_for_user(&self, user: UserId) -> RepoResult<()> {
        sqlx::query("delete from invite_tokens where user_id = $1")
            .bind(user.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }
}
