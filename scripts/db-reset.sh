#!/usr/bin/env bash
# Drop the Postgres volume, recreate the container, and re-run migrations.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

docker compose --env-file "$ROOT/.env" -f infra/docker-compose.yml down -v
bash scripts/db-up.sh
bash scripts/migrate.sh
echo "Database reset and migrated."
