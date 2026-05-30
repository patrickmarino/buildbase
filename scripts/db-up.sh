#!/usr/bin/env bash
# Bring up the local Postgres and wait until it is accepting connections.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

docker compose --env-file "$ROOT/.env" -f infra/docker-compose.yml up -d

echo "Waiting for Postgres to be healthy…"
for i in $(seq 1 30); do
  if docker compose --env-file "$ROOT/.env" -f infra/docker-compose.yml exec -T postgres pg_isready -U "${POSTGRES_USER:-core}" -d "${POSTGRES_DB:-core}" >/dev/null 2>&1; then
    echo "Postgres is ready."
    exit 0
  fi
  sleep 1
done
echo "Postgres did not become ready in time." >&2
exit 1
