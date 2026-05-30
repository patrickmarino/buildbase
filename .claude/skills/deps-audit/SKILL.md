---
name: deps-audit
description: Audit dependencies for known vulnerabilities on both sides of the stack — `cargo audit` (Rust) and `pnpm audit` (JS) — attempt safe in-range fixes, then run the test suites to verify nothing broke. Use when the user says "deps-audit", "audit deps", "check for vulnerabilities", "security audit the deps", "scan for CVEs", or wants a pre-release dependency safety check.
version: 1.0.0
---

# deps-audit — Dependency Vulnerability Audit + Fix + Verify

Scan both the Rust crates and the JS workspaces for known advisories, attempt safe in-range fixes, then run the tests to confirm no regression. **Reporting "fixed" without running tests is the failure mode this skill exists to prevent.**

## Step 0: Anchor to the repo root (worktree-aware)

```bash
REPO_ROOT=$(git rev-parse --show-toplevel); cd "$REPO_ROOT"; git branch --show-current
```

If `node_modules/` is missing, run `pnpm install` first.

## Step 1: Audit both ecosystems

**Rust** (needs `cargo-audit`; if absent, install it: `cargo install cargo-audit`):

```bash
cd apps/api && cargo audit; cd "$REPO_ROOT"
```

**JS**:

```bash
pnpm audit
```

Capture both. Outcomes: (1) **clean** — confirm and skip to Step 3 for a green check; (2) **fixable in-range** — Step 2; (3) **needs a major bump** — list each advisory (severity, package, advisory URL, fixed version) and **ask the user** before any major bump (that's a breaking-change decision). Record counts by severity for the report.

## Step 2: Attempt safe (in-range) fixes

- **JS**: `pnpm update` bumps deps to the latest release within their declared ranges (updates `pnpm-lock.yaml`). Do **not** use `pnpm update --latest` (crosses majors). Re-run `pnpm audit` and confirm the count dropped.
- **Rust**: `cargo update` refreshes `Cargo.lock` within the declared semver ranges; re-run `cargo audit`. Crates needing a major bump must be raised in `Cargo.toml` deliberately — surface them, don't auto-bump.

Note what changed:

```bash
git status --short && git diff --stat Cargo.lock pnpm-lock.yaml
```

## Step 3: Verify with the test suites

Dependency bumps can break behavior even when types/builds still resolve.

```bash
# Rust (no DB): fast and deterministic
cd apps/api && cargo test -p core-domain -p core-app && cd "$REPO_ROOT"
# Frontend
pnpm --filter core test
```

If repo/HTTP code or its deps changed, also run the integration tests with Postgres up (`pnpm db:up`, `DATABASE_URL=… cargo test`). If a single upgrade caused a regression, revert just that entry in the lockfile and re-run the install, then report the unfixable advisory. Do **not** mask failures by skipping tests.

## Step 4: Report (tight — a few lines)

1. **Before**: counts by severity per ecosystem, or "clean".
2. **Fix outcome**: how many auto-fixed, how many remain (with package + reason, e.g. "needs major bump 4.x → 5.x").
3. **Tests**: pass/fail; name failing files if any.
4. **Files changed**: `Cargo.lock`, `pnpm-lock.yaml`, and any manifests touched.

## Deliberately does NOT

- Bump majors automatically (breaking-change decision → dedicated PR + human review).
- Upgrade non-vulnerable packages for freshness (audit is about CVEs).
- Commit, push, or open a PR — pair with `/git-ship`.
- Edit source to work around a vulnerable dep — surface it and let the user decide.
