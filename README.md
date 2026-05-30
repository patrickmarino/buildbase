# Core — MadeSpace Admin

The MadeSpace identity & RBAC admin module: identities, roles & permissions, organization
settings, audit, and service-account API keys. A React + TypeScript SPA backed by an Axum (Rust)
API built with clean architecture (Postgres + sqlx, session auth).

## Layout

```
apps/
  api/    Rust Cargo workspace (Axum) — clean architecture: core-domain → core-app → core-infra → core-web
  core/   Vite + React + TypeScript SPA
e2e/      Playwright cross-stack smoke tests
infra/    docker-compose (Postgres), Dockerfiles
scripts/  db + migration helpers
```

The four Rust crates enforce clean architecture by construction: `core-domain` (pure entities,
ports, and business rules) depends on nothing infrastructural; `core-app` orchestrates use-cases
over the ports; `core-infra` implements them with sqlx/argon2; `core-web` exposes Axum HTTP.

## Prerequisites

- Rust (stable, 1.92+), `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features rustls,postgres`)
- Node 20+ and `pnpm` (`npm i -g pnpm`)
- Docker (for local Postgres)

## Quick start

```bash
cp .env.example .env
pnpm install            # JS deps for apps/core + e2e
pnpm dev                # Postgres (docker) + API + frontend, one command
```

The API auto-runs migrations and seeds a default Owner on first boot (see `SEED_OWNER_*` in `.env`).
Frontend: http://localhost:5173 · API: http://localhost:8080.

## Common commands

| Command | What it does |
|---|---|
| `pnpm dev` | Postgres + API + frontend together |
| `pnpm test` | All unit/use-case/component tests |
| `pnpm lint` | clippy + eslint |
| `pnpm db:up` / `pnpm db:reset` | Start / wipe + remigrate Postgres |
| `pnpm migrate` | Run sqlx migrations |
| `pnpm e2e` | Playwright cross-stack smoke |

### Backend only

```bash
cd apps/api
cargo test -p core-domain -p core-app    # pure + use-case tests, no DB
cargo test                                # + sqlx integration tests (needs DATABASE_URL)
cargo run -p core-web                      # serve the API
```
