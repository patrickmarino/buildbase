use super::mappers::{mfa_enforce_str, mfa_method_str, org_from_row, sso_provider_str};
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::Organization;
use core_domain::ids::{OrgId, RoleId, UserId};
use core_domain::ports::{OrgRepo, RepoResult};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PgOrgRepo {
    pool: PgPool,
}

impl PgOrgRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(crate) const ORG_COLS: &str = "id, name, domain, owner_id, pending_owner_id, branding_accent, \
    mfa_enabled, mfa_method, mfa_enforce, pw_min_length, pw_require_number, pw_require_symbol, \
    pw_rotation_days, sso_enabled, sso_provider, sso_url";

#[async_trait]
impl OrgRepo for PgOrgRepo {
    async fn get(&self, org: OrgId) -> RepoResult<Option<Organization>> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!("select {ORG_COLS} from organizations where id = $1")))
            .bind(org.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;
        row.as_ref().map(org_from_row).transpose()
    }

    async fn update(&self, org: &Organization) -> RepoResult<()> {
        sqlx::query(
            "update organizations set name=$2, domain=$3, owner_id=$4, pending_owner_id=$5, \
             branding_accent=$6, mfa_enabled=$7, mfa_method=$8, mfa_enforce=$9, \
             pw_min_length=$10, pw_require_number=$11, pw_require_symbol=$12, pw_rotation_days=$13, \
             sso_enabled=$14, sso_provider=$15, sso_url=$16 where id=$1",
        )
        .bind(org.id.as_uuid())
        .bind(&org.name)
        .bind(&org.domain)
        .bind(org.owner_id.as_uuid())
        .bind(org.pending_owner_id.map(|u| u.as_uuid()))
        .bind(&org.branding.accent_color)
        .bind(org.mfa.enabled)
        .bind(mfa_method_str(org.mfa.method))
        .bind(mfa_enforce_str(org.mfa.enforce))
        .bind(i16::from(org.password_policy.min_length))
        .bind(org.password_policy.require_number)
        .bind(org.password_policy.require_symbol)
        .bind(org.password_policy.rotation_days.map(|d| d as i16))
        .bind(org.sso.enabled)
        .bind(sso_provider_str(org.sso.provider))
        .bind(&org.sso.url)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn complete_ownership_transfer(
        &self,
        org: OrgId,
        new_owner: UserId,
        previous_owner: UserId,
        owner_role: RoleId,
        admin_role: RoleId,
    ) -> RepoResult<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        sqlx::query("update users set role_id = $2 where id = $1")
            .bind(new_owner.as_uuid())
            .bind(owner_role.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;
        sqlx::query("update users set role_id = $2 where id = $1")
            .bind(previous_owner.as_uuid())
            .bind(admin_role.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;
        sqlx::query("update organizations set owner_id = $2, pending_owner_id = null where id = $1")
            .bind(org.as_uuid())
            .bind(new_owner.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx)?;
        tx.commit().await.map_err(map_sqlx)?;
        Ok(())
    }

    async fn delete(&self, org: OrgId) -> RepoResult<()> {
        sqlx::query("delete from organizations where id = $1")
            .bind(org.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }
}
