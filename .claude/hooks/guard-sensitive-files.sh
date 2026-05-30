#!/bin/bash
# Guard sensitive files from unauthorized edits.
# - BLOCKS: .env files, credentials, secrets, key files
# - WARNS:  security-sensitive paths (authz, password hashing, sessions, migrations)
#
# Used as a Claude Code PreToolUse hook for the Edit|Write tools.
# Requires: jq (degrades to a no-op if jq is unavailable).

set -euo pipefail

command -v jq >/dev/null 2>&1 || exit 0

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

[ -z "$FILE_PATH" ] && exit 0

# --- BLOCK: Secrets and environment files ---
case "$FILE_PATH" in
  *.env|*.env.*|*/.env|*/.env.*)
    # Allow .env.example and .env.*.example templates.
    case "$FILE_PATH" in
      *.example) exit 0 ;;
    esac
    echo "Blocked: Cannot modify environment files. Use .env.example for templates." >&2
    exit 2
    ;;
  *credentials*|*secret*|*.pem|*.key)
    echo "Blocked: Cannot modify credential/secret/key files." >&2
    exit 2
    ;;
  */keys/*)
    echo "Blocked: Cannot modify files in a keys/ directory." >&2
    exit 2
    ;;
esac

# --- WARN: security-sensitive code (authz, crypto, sessions, schema) ---
SECURITY_PATHS=(
  "crates/core-domain/src/services/authz.rs"
  "crates/core-domain/src/services/role_guards.rs"
  "crates/core-infra/src/argon2_hasher.rs"
  "crates/core-infra/src/rand_token.rs"
  "crates/core-web/src/extractors.rs"
  "crates/core-web/src/cookies.rs"
  "apps/api/migrations/"
  "apps/core/src/lib/api.ts"
)

for path in "${SECURITY_PATHS[@]}"; do
  if [[ "$FILE_PATH" == *"$path"* ]]; then
    cat <<EOF
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "additionalContext": "WARNING: Security-sensitive file ($path) — authorization, password hashing, sessions, or the DB schema. Changes here can introduce auth bypasses or data-loss migrations. Proceed with extra scrutiny and prefer adding tests."
  }
}
EOF
    exit 0
  fi
done

exit 0
