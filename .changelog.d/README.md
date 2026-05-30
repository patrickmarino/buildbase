# Changelog fragments

Each PR drops one small fragment here instead of editing `.claude/docs/CHANGELOG.md`
directly — so concurrent PRs never conflict on the changelog. `/changelog-release`
(`pnpm changelog:release <version>`) aggregates every fragment into a new versioned
section and deletes the consumed fragments.

## Format

Filename: kebab-case, ideally the branch name — e.g. `feat-token-rotation.md`.

```markdown
---
type: <Added|Changed|Deprecated|Removed|Fixed|Security>
---

- Concise, present-tense bullet describing the change.
- Optional second bullet.
```

One `type` per file (Keep-a-Changelog categories). If a PR spans multiple types,
write multiple fragments. No version numbers or dates — the aggregator adds those.
