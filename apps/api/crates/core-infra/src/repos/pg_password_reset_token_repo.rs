use super::mappers::password_reset_token_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::PasswordResetToken;
use core_domain::ids::{PasswordResetTokenId, UserId};
use core_domain::ports::{PasswordResetTokenRepo, RepoResult};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgPasswordResetTokenRepo {
    pool: PgPool,
}

impl PgPasswordResetTokenRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordResetTokenRepo for PgPasswordResetTokenRepo {
    async fn insert(&self, token: &PasswordResetToken) -> RepoResult<()> {
        sqlx::query(
            "insert into password_reset_tokens (id, user_id, token_hash, created_at, expires_at) \
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

    async fn find_by_hash(&self, token_hash: &str) -> RepoResult<Option<PasswordResetToken>> {
        let row = sqlx::query(
            "select id, user_id, token_hash, created_at, expires_at \
             from password_reset_tokens where token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        row.as_ref().map(password_reset_token_from_row).transpose()
    }

    async fn delete(&self, id: PasswordResetTokenId) -> RepoResult<()> {
        sqlx::query("delete from password_reset_tokens where id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }

    async fn delete_for_user(&self, user: UserId) -> RepoResult<()> {
        sqlx::query("delete from password_reset_tokens where user_id = $1")
            .bind(user.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }
}
