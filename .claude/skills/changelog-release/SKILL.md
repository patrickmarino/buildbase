---
name: changelog-release
description: Aggregate per-PR fragments from `.changelog.d/` into `.claude/docs/CHANGELOG.md` under a new versioned section, then commit. Use when the user asks to "release the changelog", "cut a changelog version", "bump CHANGELOG", "aggregate fragments", or "publish changelog X.Y.Z".
version: 1.0.0
---

# changelog-release — Aggregate fragments into a versioned CHANGELOG entry

Per-PR changelog fragments live under `.changelog.d/`. This skill collapses them into a single new `## [<version>] - <YYYY-MM-DD>` block at the top of `.claude/docs/CHANGELOG.md`, deletes the consumed fragments, and commits.

## When to run

- The user asks to cut a changelog release / bump CHANGELOG to X.Y.Z / publish pending fragments.
- Run from a clean working tree (unrelated uncommitted changes → stop and ask first).

## Workflow

### 1. Pick the version

Latest released version:

```bash
grep -m1 -E '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' .claude/docs/CHANGELOG.md
```

Bump with semver against the pending fragments: **MAJOR** if breaking `Changed`/`Removed`; **MINOR** if any `Added`; **PATCH** if only `Fixed`/`Security`/docs. If the user named a version, use it.

### 2. Show the plan, then aggregate

List the pending fragments (`ls .changelog.d/`) and the planned version, then run:

```bash
pnpm changelog:release <version>
```

The aggregator (`scripts/changelog-release.mjs`): validates the version, reads every `.md` in `.changelog.d/` (sorted), groups bullets by `type:` in Keep-a-Changelog order (Added, Changed, Deprecated, Removed, Fixed, Security), prepends the new section to `.claude/docs/CHANGELOG.md`, and removes the consumed fragments. It exits non-zero on: no fragments, version already released, malformed frontmatter, or unknown `type:`.

### 3. Review & commit

```bash
git diff -- .claude/docs/CHANGELOG.md
git add -A
git commit -m "📝 Docs(changelog): Aggregate fragments for v<version>"
git push
```

End the commit message with the standard `Co-Authored-By` trailer.

## Failure modes

| Situation | Action |
|---|---|
| `no changelog fragments to release` | Nothing to do — verify with the user before fabricating entries. |
| `version X is already released` | Pick the next bump, or edit the file by hand to amend. |
| `unknown type "<x>"` | Fix the fragment's `type:`, re-run. |
| `missing or malformed frontmatter` | Wrap the fragment per `.changelog.d/README.md`. |
| Working tree dirty with unrelated changes | Stop. Ask the user to commit/stash first. |

## Verify

```bash
head -20 .claude/docs/CHANGELOG.md   # new section at top
ls .changelog.d/                     # only README.md remains
```
