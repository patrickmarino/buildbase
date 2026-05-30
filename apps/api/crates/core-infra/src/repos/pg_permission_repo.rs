use super::mappers::cell_state_from_row;
use crate::error::map_sqlx;
use async_trait::async_trait;
use core_domain::entities::{MatrixCell, PermissionMatrix, PermissionState};
use core_domain::ids::{OrgId, RoleId};
use core_domain::ports::{PermissionRepo, RepoResult};
use core_domain::seed;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct PgPermissionRepo {
    pool: PgPool,
}

impl PgPermissionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepo for PgPermissionRepo {
    async fn load_matrix(&self, org: OrgId) -> RepoResult<PermissionMatrix> {
        // Column roles + ranks, most → least powerful.
        let role_rows = sqlx::query(
            "select id, rank from roles where org_id = $1 and is_column = true order by rank desc",
        )
        .bind(org.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        let roles = role_rows
            .iter()
            .map(|r| {
                let id: uuid::Uuid = r.try_get("id").map_err(map_sqlx)?;
                let rank: i16 = r.try_get("rank").map_err(map_sqlx)?;
                Ok((RoleId::from_uuid(id), rank))
            })
            .collect::<RepoResult<Vec<_>>>()?;

        let cell_rows = sqlx::query(
            "select role_id, action_key, state from permission_cells where org_id = $1",
        )
        .bind(org.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        let cells = cell_rows
            .iter()
            .map(|r| {
                let (action_key, role_id, state) = cell_state_from_row(r)?;
                Ok(MatrixCell { action_key, role_id, state })
            })
            .collect::<RepoResult<Vec<_>>>()?;

        Ok(PermissionMatrix {
            groups: seed::default_groups(),
            roles,
            cells,
        })
    }

    async fn set_cell(
        &self,
        org: OrgId,
        action_key: &str,
        role: RoleId,
        state: PermissionState,
    ) -> RepoResult<()> {
        sqlx::query(
            "insert into permission_cells (org_id, role_id, action_key, state) \
             values ($1, $2, $3, $4) \
             on conflict (role_id, action_key) do update set state = excluded.state",
        )
        .bind(org.as_uuid())
        .bind(role.as_uuid())
        .bind(action_key)
        .bind(state.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn add_role_with_copied_cells(
        &self,
        org: OrgId,
        new_role: RoleId,
        base: RoleId,
    ) -> RepoResult<()> {
        sqlx::query(
            "insert into permission_cells (org_id, role_id, action_key, state) \
             select $1, $2, action_key, state from permission_cells where role_id = $3 \
             on conflict (role_id, action_key) do nothing",
        )
        .bind(org.as_uuid())
        .bind(new_role.as_uuid())
        .bind(base.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }
}
