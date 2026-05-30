---
description: From-scratch setup wizard — ask for project name + database + basics, then configure and set up the whole repo (api + core).
---

Set this repo up from scratch as if it's a brand-new project. First **gather the configuration**, then run the automated setup in `scripts/bootstrap.sh`, then verify.

## 1. Collect the settings (suggest sensible defaults)

Ask the user for the following, showing the suggested default for each and letting them accept all defaults at once. Derive suggestions from the project name they give (e.g. a database name is the snake_case of the project name):

| Setting | Suggested default |
|---|---|
| **Project name** | `Core Admin` |
| **Postgres database** | snake_case of the project name (e.g. `core_admin`) |
| **Postgres user** | same as the database name |
| **Postgres password** | `devpassword` |
| **Postgres host port** | `5440` (5432–5434 are usually taken locally) |
| **Organization name** | the project name |
| **Organization domain** | `example.com` |
| **Seed owner email** | `owner@<domain>` |
| **Seed owner password** | `changeme-dev-only` (recommend they change it) |

Use AskUserQuestion (or a short conversational prompt) to confirm the project name and offer "use all suggested defaults" vs. "customize". Keep it to one or two quick rounds — don't interrogate.

## 2. Run the setup (non-interactive, passing the answers as env vars)

```
FORCE_CONFIG=1 \
PROJECT_NAME="<name>" DB_NAME="<db>" DB_USER="<user>" DB_PASSWORD="<pass>" DB_PORT="<port>" \
ORG_NAME="<org>" ORG_DOMAIN="<domain>" OWNER_EMAIL="<email>" OWNER_PASSWORD="<pass>" \
pnpm bootstrap
```

The script writes `.env`, renames the root package to the slugified project name, then installs JS deps, readies the Rust toolchain, starts Postgres (docker), runs migrations, and builds the backend. It backs up any existing `.env` to `.env.bak`.

If a step fails, fix it rather than stopping. Common cases:
- **Port already allocated** → pick a free `DB_PORT` and re-run.
- **DB name/credentials changed but a Postgres volume exists** → run `pnpm db:reset` (wipes the volume so the new database/user are created), then re-run bootstrap.
- **`expected to read 5 bytes, got 0 at EOF`** → the DB wasn't ready or `DATABASE_URL` lacks `?sslmode=disable` (the template includes it).
- **rustc too old for sqlx 0.9** → `rustup update stable` (repo pins 1.96).
- **Docker not running** → ask the user to start Docker Desktop.

## 3. Verify and report

Smoke-test without leaving servers running: from `apps/api`, run `cargo test -p core-domain -p core-app`.

Then report: the chosen project/database names, the seeded Owner credentials, and that `pnpm dev` now starts Postgres + API + frontend (frontend :5173, API :8080). Don't start `pnpm dev` yourself unless asked.
