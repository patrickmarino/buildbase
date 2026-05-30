#!/usr/bin/env node
// Aggregate per-PR fragments from `.changelog.d/` into a new versioned section
// at the top of `.claude/docs/CHANGELOG.md`, then remove the consumed fragments.
//
// Usage: node scripts/changelog-release.mjs <version>   (e.g. 0.2.0)
// Exits non-zero on: bad version, no fragments, version already released,
// malformed frontmatter, or unknown type.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const FRAG_DIR = path.join(ROOT, ".changelog.d");
const CHANGELOG = path.join(ROOT, ".claude", "docs", "CHANGELOG.md");
const ORDER = ["Added", "Changed", "Deprecated", "Removed", "Fixed", "Security"];

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error("error: provide a semver version, e.g. `node scripts/changelog-release.mjs 0.2.0`");
  process.exit(1);
}

const existing = fs.existsSync(CHANGELOG) ? fs.readFileSync(CHANGELOG, "utf8") : "# Changelog\n";
if (existing.includes(`## [${version}]`)) {
  console.error(`error: version ${version} is already released`);
  process.exit(1);
}

const fragments = fs
  .readdirSync(FRAG_DIR)
  .filter((f) => f.endsWith(".md") && f !== "README.md")
  .sort();
if (fragments.length === 0) {
  console.error("error: no changelog fragments to release in .changelog.d/");
  process.exit(1);
}

const buckets = Object.fromEntries(ORDER.map((t) => [t, []]));
for (const file of fragments) {
  const raw = fs.readFileSync(path.join(FRAG_DIR, file), "utf8");
  const m = raw.match(/^---\s*\ntype:\s*(\w+)\s*\n---\s*\n([\s\S]*)$/);
  if (!m) {
    console.error(`error: ${file}: missing or malformed frontmatter (--- type: <Type> ---)`);
    process.exit(1);
  }
  const type = m[1];
  if (!ORDER.includes(type)) {
    console.error(`error: ${file}: unknown type "${type}"`);
    process.exit(1);
  }
  for (const line of m[2].split("\n")) {
    if (line.trim().startsWith("-")) buckets[type].push(line.trim());
  }
}

const date = new Date().toISOString().slice(0, 10);
let section = `## [${version}] - ${date}\n\n`;
for (const type of ORDER) {
  if (buckets[type].length === 0) continue;
  section += `### ${type}\n\n${buckets[type].join("\n")}\n\n`;
}

// Insert above the first existing version block; otherwise after the H1.
const lines = existing.split("\n");
let insertAt = lines.findIndex((l) => /^## \[/.test(l));
if (insertAt < 0) {
  const headIdx = lines.findIndex((l) => l.startsWith("# "));
  insertAt = headIdx >= 0 ? headIdx + 1 : 0;
}
lines.splice(insertAt, 0, section.trimEnd(), "");
fs.writeFileSync(CHANGELOG, lines.join("\n").replace(/\n{3,}/g, "\n\n"));

for (const file of fragments) fs.rmSync(path.join(FRAG_DIR, file));

console.log(`Aggregated ${fragments.length} fragment(s) into v${version}.`);
console.log("Removed:", fragments.join(", "));
console.log("Now review and commit: git add -A && git commit -m \"📝 Docs(changelog): Aggregate fragments for v" + version + "\"");
