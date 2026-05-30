//! Postgres connection pool and migrations.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Build a connection pool. `max_connections` is modest by default — tune via
/// the caller for production.
pub async fn connect(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

/// Run all pending migrations. The path is resolved at compile time relative to
/// this crate, pointing at `apps/api/migrations`.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}
