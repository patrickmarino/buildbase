# Security reference

This file lists the security-sensitive surfaces of the codebase and the invariants
that must hold. The `guard-sensitive-files.sh` hook warns when these paths are edited;
treat changes here with extra scrutiny and always add tests.

## Security-sensitive files

| Area | File(s) | Invariant |
|---|---|---|
| Authorization | `apps/api/crates/core-domain/src/services/authz.rs` | `can()` is **deny-by-default**: unknown action / missing cell ⇒ Deny; `Scope` ⇒ Allow only on the actor's own resource. |
| Role guards | `apps/api/crates/core-domain/src/services/role_guards.rs` | No privilege escalation (cannot assign above own rank); last active admin/owner cannot be removed or demoted. |
| Password hashing | `apps/api/crates/core-infra/src/argon2_hasher.rs` | Argon2id with a per-password CSPRNG salt; verify is constant-time; never store plaintext. |
| Tokens / sessions | `apps/api/crates/core-infra/src/rand_token.rs` | Session ids and API tokens come from the OS CSPRNG (`getrandom`); API tokens are stored only as a SHA-256 hash (+ display prefix). |
| Session auth | `apps/api/crates/core-web/src/extractors.rs`, `cookies.rs` | The `core_sid` cookie is `HttpOnly; SameSite=Lax` (Secure in prod). `CurrentUser` resolution rejects invalid/expired sessions and deactivated users. |
| DB schema | `apps/api/migrations/`, `apps/api/crates/core-domain/src/seed.rs` | Migrations are additive; destructive changes need review. Seed defaults live in `seed.rs` (single source of truth). |
| Web API client | `apps/core/src/lib/api.ts` | All requests send the session cookie (`credentials: "include"`); never put tokens in localStorage or URLs. |

## Enforced invariants (what reviewers must verify)

- **Deny-by-default authorization.** Every mutating use-case calls `ActorContext::require(action)` before acting. A new data endpoint without an authz check is a bug.
- **Audit every sensitive action.** Mutations append an `AuditEvent` (actor, action, target, before/after, ip) via the `Auditor`.
- **Session revocation.** Deactivating a user or changing their role calls `SessionRepo::delete_all_for_user` — access must drop immediately.
- **No secret leakage.** `password_hash` and `token_hash` are `#[serde(skip)]` and never appear in a DTO. The one-time API token is returned only on creation, never re-fetchable.
- **Parameterized SQL.** Use `sqlx::query`/`QueryBuilder` bind parameters. The only dynamic SQL allowed is trusted const column lists wrapped in `sqlx::AssertSqlSafe(...)` — never interpolate user input.
- **No TLS-less secrets in prod.** Local dev connects to Postgres without TLS (`?sslmode=disable`); production should use a TLS connection and `COOKIE_SECURE=true`.

## Things that block in tooling

- `.env` / `*.key` / `*.pem` / `*credentials*` / `*secret*` edits are **blocked** by the file guard (use `.env.example`).
- Force push, hard reset, force clean, `rm -rf /|~|$HOME`, and `DROP TABLE/DATABASE`/`TRUNCATE TABLE` are **blocked** by the command guard.

## Review focus for auth/RBAC changes

When reviewing changes to the files above, prioritize: privilege-escalation paths, cross-org/tenant scoping, timing/enumeration on login (failures collapse to a generic `Unauthorized`), session fixation/revocation, and that new permissions are reflected in both the matrix seed and the `/auth/me` permission list the SPA gates on.
