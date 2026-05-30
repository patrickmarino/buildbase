#!/bin/bash
# Guard against destructive shell commands.
# - BLOCKS: force push, hard reset, force clean, rm -rf root/home, destructive SQL
#
# Used as a Claude Code PreToolUse hook for the Bash tool.
# Requires: jq (degrades to a no-op if jq is unavailable).

set -euo pipefail

command -v jq >/dev/null 2>&1 || exit 0

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

[ -z "$COMMAND" ] && exit 0

# --- BLOCK: Destructive git operations ---
if echo "$COMMAND" | grep -qE "git\s+push\s+.*--force|git\s+push\s+.*-f\b"; then
  echo "Blocked: Force push is not allowed. Use a standard push instead." >&2
  exit 2
fi

if echo "$COMMAND" | grep -qE "git\s+reset\s+--hard"; then
  echo "Blocked: Hard reset is not allowed. Use git stash or git checkout for specific files." >&2
  exit 2
fi

if echo "$COMMAND" | grep -qE "git\s+clean\s+-f"; then
  echo "Blocked: Force clean is not allowed." >&2
  exit 2
fi

# --- BLOCK: Destructive filesystem operations ---
if echo "$COMMAND" | grep -qE "rm\s+-rf\s+/|rm\s+-rf\s+~|rm\s+-rf\s+\\\$HOME"; then
  echo "Blocked: Recursive deletion of root/home directories is not allowed." >&2
  exit 2
fi

# --- BLOCK: Destructive SQL operations ---
# Migrations belong in apps/api/migrations and run via sqlx; ad-hoc destructive
# SQL against a live database needs explicit human approval.
if echo "$COMMAND" | grep -iqE "DROP\s+TABLE|DROP\s+DATABASE|TRUNCATE\s+TABLE"; then
  echo "Blocked: Destructive SQL commands require explicit human approval." >&2
  exit 2
fi

exit 0
