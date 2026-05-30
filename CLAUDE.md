# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**Core** — the MadeSpace identity & RBAC admin module (users, roles & permissions, org settings, audit, service-account API keys). A React + TypeScript SPA backed by an Axum (Rust) API. It was implemented from a Claude Design handoff bundle; `apps/core/src/styles/*.css` are ported verbatim from that bundle and are the pixel spec — recreate visuals against those classes, don't reinvent them.

Turborepo + pnpm monorepo:
- `apps/api/` — Rust Cargo workspace (Axum), clean architecture.
- `apps/core/` — Vite + React + TS SPA.
- `e2e/` — Playwright (drives the full stack). `infra/` — docker-compose (Postgres). `scripts/` — db helpers.

## Commands

Run from the repo root unless noted. First-time: `cp .env.example .env && pnpm install`.

| Task | Command |
|---|---|
| Everything (Postgres + API + SPA) | `pnpm dev` |
| Build / lint everything (turbo) | `pnpm build` · `pnpm lint` (clippy + eslint) |
| Start / wipe+remigrate Postgres | `pnpm db:up` · `pnpm db:reset` |
| Run migrations | `pnpm migrate` |
| Playwright e2e | `pnpm e2e` (Postgres must be up first) |

**Backend** (`cd apps/api`):
- Fast tests, no DB: `cargo test -p core-domain -p core-app`
- All tests incl. sqlx/HTTP integration: `DATABASE_URL=... cargo test` (needs Postgres; `#[sqlx::test]` creates an isolated DB per test)
- One test: `cargo test -p core-app change_role_revokes_sessions` (substring match)
- Lint: `cargo clippy --all-targets -- -D warnings`
- Serve: `cargo run -p core-web`

**Frontend** (`cd apps/core`):
- `pnpm test` (Vitest, jsdom + MSW) · single file: `pnpm exec vitest run src/lib/matrix.test.ts`
- `pnpm typecheck` · `pnpm lint` · `pnpm dev` (Vite on :5173, proxies `/api` → :8080)

## Environment (non-obvious, will bite you)

- **Postgres is on host port 5440**, not 5432 (other local DBs occupy 5432–5434). Configurable via `PG_HOST_PORT`.
- `DATABASE_URL` must include **`?sslmode=disable`** — local Postgres has no TLS; otherwise sqlx fails with `expected to read 5 bytes, got 0 at EOF` on connect.
- The Rust toolchain is pinned to **1.96** in `apps/api/rust-toolchain.toml` because **sqlx 0.9 requires rustc ≥ 1.94**.
- Single-tenant dev: login resolves the **sole seeded org**. The API seeds a default org + Owner on first boot when the DB has no users (`SEED_OWNER_*`, default `elena@madespace.co` / `changeme-dev-only`). Re-seed cleanly with `pnpm db:reset`.

## Backend architecture (the important part)

A 4-crate Cargo workspace where **dependencies point inward and the boundary is compiler-enforced** — `core-domain`'s `Cargo.toml` simply has no axum/sqlx/tokio, so infrastructure cannot leak in. Layers:

- **`core-domain`** — pure. Entities, value objects, strongly-typed IDs, the repository **traits** (ports in `ports/`), and the business rules in `services/`: `authz::can` (deny-by-default, the FR-4 authorization API), `matrix_rules` (cell cycling + lock rules), `role_guards` (no-privilege-escalation, last-admin protection, custom-role inheritance), `password_policy`. These are plain sync functions over loaded data and carry the bulk of the unit tests. `seed.rs` is the **single source of truth** for the default roles/actions/matrix (mirrors the design's `data.jsx`); both the infra startup seed and the in-memory test fakes build from it.
- **`core-app`** — use-case services (`auth`, `user`, `role`, `permission`, `org`, `audit`, `api_key`), each holding `Arc<dyn ...Repo>`. Two cross-cutting rules are applied uniformly: **authorization** via `ActorContext::require(action)` (a capability check — `Allow`/`Scope` pass; the fine-grained limit is enforced by the domain guards), and **auditing** via `Auditor` (every mutation records a before/after event). `testing.rs` (behind `test-support`/`#[cfg(test)]`) has in-memory fakes + a seeded `World` for DB-free use-case tests.
- **`core-infra`** — sqlx Postgres repo impls (`repos/`, with `mappers.rs` doing row→domain), `Argon2Hasher`, `RandTokenGenerator`, `SystemClock`, and `seed::ensure_seeded`.
- **`core-web`** — Axum. `AppState` wires concrete infra repos into the app services. `CurrentUser` is a `FromRequestParts` extractor that resolves the actor from the `core_sid` session cookie — its presence on a handler **is** the auth guard (no global middleware). DTOs (camelCase, enums as lowercase strings) live only here; `error.rs` maps `DomainError` → HTTP (403/404/409/422/500) with a stable `{ error, message }` body.

Key flow: a request → `CurrentUser` extractor (session → role → matrix loaded once) → handler → app service (authz + mutation + audit) → infra repo. The `ActorContext` carries the matrix so authorization is synchronous.

### Conventions when editing the backend
- Repository methods are async (`async-trait`); pure domain services are sync. New business rules go in `core-domain/services` with unit tests, not in handlers.
- sqlx 0.9 rejects dynamic `&String` SQL; trusted `format!`-with-const-columns queries are wrapped in `sqlx::AssertSqlSafe(...)`. Queries are runtime-checked (`sqlx::query`/`QueryBuilder`), not the `query!` macro, so no compile-time DB is needed.
- Randomness (salts, tokens) uses the `getrandom` crate directly, not argon2's feature-gated `OsRng` re-export.
- Domain enums are stored as `TEXT` and round-tripped via inherent `as_str`/`from_str` (intentionally not `std::str::FromStr`).

## Frontend architecture

- `lib/api.ts` — typed fetch client; always `credentials: "include"`, throws `ApiError` carrying the backend's stable `code`. `lib/matrix.ts` / `lib/format.ts` are pure (unit-tested).
- `store/AppContext.tsx` — the only store: signed-in actor (`me`), `can(action)` gating (from `me.permissions`), toasts, brand accent. `App.tsx` gates on auth (Login vs. shell) and routes pages via sidebar state.
- Pages in `pages/` mirror the five product areas; shared components (`Sidebar`, `Topbar`, `Modal`, `RoleChip`, etc.) in `components/`. The matrix updates optimistically with the server's returned cell state.
- Tests use Vitest + Testing Library + **MSW** (`src/test/server.ts`); register handlers per test with `server.use(...)`. In jsdom, relative `/api` URLs resolve against `http://localhost`, so the same client works in tests.

## Working in this repo (conventions)

### Planning — test-first
Plan features Red → Green → Refactor: write the **failing test first** (it defines the behavior), then the minimum code to pass, then refactor while green. Plans for new features should list the failing tests before the implementation steps. Pure rules → unit tests in `core-domain`; use-cases → `core-app` tests with the in-memory `World`; endpoints → `#[sqlx::test]` HTTP tests; UI → Vitest+MSW for components/flows, Playwright (`e2e/`) for full cross-stack browser flows.

### DB modifications
Whenever work touches the database (schema, migration, seed, ad-hoc query against real data), target the **dockerized Postgres first** (`pnpm db:up`; it's on host port 5440). Only fall back to another database if Docker Postgres is genuinely unavailable — and surface that fallback before running it. Seed defaults live in `core-domain/src/seed.rs`, not hand-written in migrations.

### Naming
- Rust: `snake_case` modules/functions, `PascalCase` types/traits, `SCREAMING_SNAKE_CASE` consts.
- TS: PascalCase components/types, camelCase vars/fns/hooks (`useX`), UPPER_SNAKE_CASE consts; booleans `is`/`has`/`should`. No `any`; type-only imports use `type`.
- Keep frontend enum/string values aligned with backend DTOs (lowercase, e.g. `"allow"`, `"active"`).

### Security-sensitive code
Authorization, password hashing, sessions, the API client, and migrations are security-sensitive — see `.claude/docs/SECURITY.md` for the file list and invariants (deny-by-default authz, audit every mutation, session revocation, no secret leakage, parameterized SQL). The `guard-sensitive-files.sh` hook warns on edits to these.

### Agents & skills
- Subagents: **backend-developer** (Rust/Axum clean architecture), **frontend-developer** (React/Vite/MSW), **skeptic-reviewer** (adversarial security/UX review — run after a feature or plan). Each has persistent memory under `.claude/agent-memory/`.
- Skills: `/bootstrap` (from-scratch setup wizard), `/git-ship` (atomic commits + verify + changelog fragment + PR), `/changelog-release` (aggregate `.changelog.d/` fragments → `CHANGELOG.md`), `/deps-audit` (`cargo audit` + `pnpm audit` + verify).
- Changelog: never edit `.claude/docs/CHANGELOG.md` per PR — drop a fragment in `.changelog.d/` (see its README); `/changelog-release` aggregates.

### Guardrails (enforced by hooks in `.claude/settings.json`)
Blocked: editing `.env`/secret/key files (use `.env.example`); force push, hard reset, force clean, `rm -rf /|~|$HOME`, and destructive SQL (`DROP TABLE/DATABASE`, `TRUNCATE`). Personal allowances (docker, brew) go in `settings.local.json` (gitignored — copy from the `.example`).
