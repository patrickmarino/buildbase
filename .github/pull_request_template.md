## Summary

<!-- What does this PR do and why? -->

## Changes

<!-- Bullet the notable changes (commit-level). -->
-

## Test plan

<!-- How was this verified? Reference the suites you ran. -->
- [ ] `cargo test -p core-domain -p core-app` (fast, no DB)
- [ ] `cargo test` with Postgres up (sqlx/HTTP integration) — if backend touched
- [ ] `pnpm --filter core test` / `typecheck` / `lint` — if frontend touched
- [ ] `pnpm e2e` — if a cross-stack flow changed

## Checklist

- [ ] Mutations are authorized (deny-by-default) and audited
- [ ] No secrets/tokens leaked in DTOs, logs, or commits
- [ ] A changelog fragment was added under `.changelog.d/` (if user-facing)
- [ ] Security-sensitive changes reviewed against `.claude/docs/SECURITY.md`
