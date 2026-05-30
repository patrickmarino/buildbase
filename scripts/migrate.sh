#!/usr/bin/env bash
# Run sqlx migrations against DATABASE_URL.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Pull DATABASE_URL out of .env without sourcing the whole file (values may
# contain spaces, which a naive `source` would choke on).
if [[ -z "${DATABASE_URL:-}" && -f "$ROOT/.env" ]]; then
  export DATABASE_URL="$(grep -E '^DATABASE_URL=' "$ROOT/.env" | head -1 | cut -d= -f2-)"
fi

cd "$ROOT/apps/api"
sqlx migrate run --source migrations
