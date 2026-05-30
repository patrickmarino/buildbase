use super::mappers::user_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::email::Email;
use core_domain::entities::User;
use core_domain::ids::{OrgId, UserId};
use core_domain::ports::{RepoResult, UserFilter, UserRepo};
use sqlx::{PgPool, QueryBuilder, Row};

#[derive(Clone)]
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const COLS: &str =
    "id, org_id, email, name, role_id, status, scope, password_hash, created_at, last_active_at";

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn find_by_id(&self, id: UserId) -> RepoResult<Option<User>> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!("select {COLS} from users where id = $1")))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;
        row.as_ref().map(user_from_row).transpose()
    }

    async fn find_by_email(&self, org: OrgId, email: &Email) -> RepoResult<Option<User>> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "select {COLS} from users where org_id = $1 and email = $2"
        )))
        .bind(org.as_uuid())
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        row.as_ref().map(user_from_row).transpose()
    }

    async fn list(&self, org: OrgId, filter: &UserFilter) -> RepoResult<Vec<User>> {
        let mut qb = QueryBuilder::new(format!("select {COLS} from users where org_id = "));
        qb.push_bind(org.as_uuid());
        if let Some(role) = filter.role_id {
            qb.push(" and role_id = ").push_bind(role.as_uuid());
        }
        if let Some(status) = filter.status {
            qb.push(" and status = ").push_bind(status.as_str());
        }
        if let Some(q) = &filter.query {
            let pat = format!("%{}%", q.to_lowercase());
            qb.push(" and (lower(name) like ")
                .push_bind(pat.clone())
                .push(" or email like ")
                .push_bind(pat)
                .push(")");
        }
        qb.push(" order by name asc");

        let rows = qb.build().fetch_all(&self.pool).await.map_err(map_sqlx)?;
        rows.iter().map(user_from_row).collect()
    }

    async fn insert(&self, user: &User) -> RepoResult<()> {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into users ({COLS}) values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
        )))
        .bind(user.id.as_uuid())
        .bind(user.org_id.as_uuid())
        .bind(user.email.as_str())
        .bind(&user.name)
        .bind(user.role_id.as_uuid())
        .bind(user.status.as_str())
        .bind(&user.scope)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.last_active_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn update(&self, user: &User) -> RepoResult<()> {
        sqlx::query(
            "update users set email=$2, name=$3, role_id=$4, status=$5, scope=$6, \
             password_hash=$7, last_active_at=$8 where id=$1",
        )
        .bind(user.id.as_uuid())
        .bind(user.email.as_str())
        .bind(&user.name)
        .bind(user.role_id.as_uuid())
        .bind(user.status.as_str())
        .bind(&user.scope)
        .bind(&user.password_hash)
        .bind(user.last_active_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn count_active_admins(&self, org: OrgId) -> RepoResult<u64> {
        let row = sqlx::query(
            "select count(*) as n from users u join roles r on u.role_id = r.id \
             where u.org_id = $1 and u.status = 'active' and r.builtin in ('owner','admin')",
        )
        .bind(org.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;
        let n: i64 = row.try_get("n").map_err(map_sqlx)?;
        Ok(n.max(0) as u64)
    }
}
