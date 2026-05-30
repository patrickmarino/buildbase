//! Core API server entrypoint: load env → connect Postgres → migrate → seed →
//! build the router → serve.

use anyhow::Context;
use core_infra::seed::{ensure_seeded, sole_org_id, SeedConfig};
use core_infra::{connect, migrate, Argon2Hasher};
use core_web::{build_router, AppState, WebConfig};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,core_web=debug".into()))
        .init();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = connect(&database_url, 10).await.context("connecting to Postgres")?;
    migrate(&pool).await.context("running migrations")?;

    if env_flag("SEED_OWNER", true) {
        let cfg = seed_config();
        let hasher = Argon2Hasher;
        if ensure_seeded(&pool, &hasher, &cfg).await.context("seeding")? {
            tracing::info!("seeded default organization and owner <{}>", cfg.owner_email);
        }
    }

    let default_org = sole_org_id(&pool)
        .await?
        .context("no single organization found — seed the database first")?;

    let cfg = WebConfig::from_env();
    let bind = cfg.bind_addr.clone();
    let state = AppState::new(pool, cfg, default_org);
    let app = build_router(state);

    let listener = TcpListener::bind(&bind).await.with_context(|| format!("binding {bind}"))?;
    tracing::info!("core-web listening on http://{bind}");
    axum::serve(listener, app).await.context("server error")?;
    Ok(())
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(default)
}

fn seed_config() -> SeedConfig {
    let owner_email =
        std::env::var("SEED_OWNER_EMAIL").unwrap_or_else(|_| "elena@madespace.co".into());
    let owner_name = owner_email
        .split('@')
        .next()
        .map(|s| {
            s.replace(['.', '_'], " ")
                .split_whitespace()
                .map(capitalize)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Studio Owner".into());

    SeedConfig {
        org_name: std::env::var("SEED_ORG_NAME").unwrap_or_else(|_| "MadeSpace Studio".into()),
        org_domain: std::env::var("SEED_ORG_DOMAIN").ok(),
        owner_email,
        owner_name,
        owner_password: std::env::var("SEED_OWNER_PASSWORD")
            .unwrap_or_else(|_| "changeme-dev-only".into()),
    }
}

fn capitalize(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
