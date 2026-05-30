---
name: git-ship
description: Organize uncommitted changes into logical, atomic commits on the current branch with gitmoji-style messages, verify build/lint/tests, write a per-PR changelog fragment, push, and open a pull request. Worktree-aware. Use when the user says "git-ship", "ship it", "organize commits", "chunk commits", "create PR", "commit and push", or similar.
version: 1.0.0
---

# git-ship — Commit, Verify, Push & PR

Organize uncommitted changes into logical, atomic commits on the current branch (works in the main checkout **or** a git worktree), with verification + auto-fix, a per-PR changelog **fragment** under `.changelog.d/`, push, and PR creation.

> This repo is **pnpm + cargo** (Turborepo monorepo). Use `pnpm …` and `cargo …` — never npm/yarn/bun.
> The PR base is `develop` if it exists on `origin`, otherwise `main`. Never force-push; `main` is protected (push a branch + PR).

## Step 0: Detect repo context (worktree-aware)

```bash
git rev-parse --is-inside-work-tree
REPO_ROOT=$(git rev-parse --show-toplevel)
GIT_DIR=$(git rev-parse --git-dir); GIT_COMMON_DIR=$(git rev-parse --git-common-dir)
[ "$GIT_DIR" != "$GIT_COMMON_DIR" ] && WORKTREE_MODE=1 || WORKTREE_MODE=0
git worktree list; git branch --show-current
```

When `WORKTREE_MODE=1`: `cd "$REPO_ROOT"` before any git/verify step, anchor all paths to the worktree root, and never `git checkout` a different branch. Print one line of detected context.

## Step 1: Analyze changes

From `$REPO_ROOT`: `git status --short`, `git diff --stat`, `git diff --cached --stat`, `git log --oneline -10` (learn the existing message style). Categorize files: Database (migrations, `core-domain/src/seed.rs`), Backend (crates/*), Frontend (apps/core/src), Docs, Config, Tests.

## Step 2: Plan commits

All commits stay on the current branch — never switch. Group related files into atomic commits. Present the full plan and **wait for approval**: repo context, branch, commit groupings (files + message), the proposed `.changelog.d/<slug>.md` fragment, and the PR base.

## Step 3: Verify (build → lint → tests), auto-fix, max 5 iterations

Run from `$REPO_ROOT`; stop at the first failure and fix before moving on.

1. **Build**: `pnpm build` (turbo → `cargo build` + `vite build`).
2. **Lint**: `pnpm lint` (clippy `-D warnings` + eslint `--max-warnings 0`) and `pnpm --filter core typecheck`.
3. **Tests** (fast, no DB): `cd apps/api && cargo test -p core-domain -p core-app` and `pnpm --filter core test`. If your change touched repos/HTTP/migrations, also run the integration tests with Postgres up: `pnpm db:up` then `DATABASE_URL=... cargo test`.

Fixing: read the failing file(s), apply minimal fixes, re-run the failing check. **DO** fix type/borrow errors, add missing imports, remove genuinely unused items, prefix unused with `_`. **NEVER** add `#[allow]`/`@ts-ignore`/`any` just to silence a check, delete code to dodge errors, or loosen tsconfig/clippy config. If a fix would change behavior or you loop on the same error, stop and ask. After 5 iterations without green, **stop — do not commit**.

## Step 4: Write the changelog fragment

After checks pass, write one fragment at `$REPO_ROOT/.changelog.d/<slug>.md` (slug = kebab branch name). Do **not** edit `.claude/docs/CHANGELOG.md` directly — `/changelog-release` aggregates fragments.

```markdown
---
type: <Added|Changed|Deprecated|Removed|Fixed|Security>
---

- Concise, present-tense bullet describing the change.
```

Multiple types → multiple fragment files. Fold any auto-fix churn into a sensible bullet. The fragment is the **first** commit.

## Step 5: Execute

```bash
cd "$(git rev-parse --show-toplevel)"
CURRENT_BRANCH=$(git branch --show-current)
BASE=$(git ls-remote --heads origin develop | grep -q develop && echo develop || echo main)

git add .changelog.d/<slug>.md && git commit -m "📝 Docs(changelog): Add fragment for <slug>"
# per logical group:
git add <specific-files> && git commit -m "<gitmoji> <Type>(<scope>): <Description>"

git push --set-upstream origin "$CURRENT_BRANCH"
if [ "$CURRENT_BRANCH" != "$BASE" ]; then
  gh pr create --base "$BASE" --title "<gitmoji> <Type>(<scope>): <Description>" --body "$(cat <<'EOF'
## Summary
<bullets>

## Changes
<commits>

## Test Plan
<scenarios>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
fi
```

End commit messages with the standard `Co-Authored-By` trailer. On the base branch (`develop`/`main`) push directly, no PR.

## Commit message format

`<emoji> <Type>(<scope>): <Description>` — ✨ Feat · 🐛 Fix · ♻️ Refactor · 🔒 Security · 📝 Docs · 🔧 Config · ⚡ Perf · 🗃️ DB · ✅ Test.

## Error recovery

| Situation | Action |
|---|---|
| `gh` not found | `brew install gh && gh auth login` |
| `turbo`/deps missing | `pnpm install` (inside the worktree if `WORKTREE_MODE=1`) |
| clippy fails | fix the lint; never add `#[allow]` just to pass |
| integration tests need a DB | `pnpm db:up` and export `DATABASE_URL` (with `?sslmode=disable`) |
| Merge conflicts | stop, report, ask |
| On a protected base with no branch | warn; suggest `git checkout -b <feature>` first |
| Build fails after 5 iterations | stop — do NOT commit; report remaining errors |
