# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and per-PR entries are collected
from `.changelog.d/` fragments via `pnpm changelog:release <version>`.

## [0.1.0] - 2026-05-30

### Added

- Initial release: Core — MadeSpace identity & RBAC admin (Axum + React monorepo).
- Clean-architecture Rust API (core-domain → core-app → core-infra → core-web) with
  deny-by-default authorization, session auth (Argon2), and audit logging.
- React + TypeScript SPA: login, Roles matrix, Users, Organization, Audit, API Keys.
- Turborepo + pnpm tooling; `pnpm bootstrap` / `/bootstrap` setup wizard.
