---
id: QA-103
title: '"Add More Frameworks" lists 70 chips for 30 frameworks — every name repeated two or three times'
status: verified
severity: minor
kind: bug
area: settings/ides
mode: tauri
found: 2026-11-07
---

## Use case

Pick a framework to map to an IDE from the suggestion chips.

## Steps

1. `/settings/ides`, add one IDE and one mapping so the "Add More Frameworks" block renders.
2. Count the chips.

## Expected

Each framework offered once.

## Actual

70 chips, 30 distinct names. Ten names appear three times (Node.js, Express, Fastify, NestJS,
Tauri, Electron, Flutter, React, Vue, Angular — once under RECOMMENDED, once under their own
category, once from the database row), twenty appear twice:

```
{"total":70,"dupes":[["Node.js",3],["Express",3],["Fastify",3],["NestJS",3],["Tauri",3],
 ["Electron",3],["Flutter",3],["React",3],["Vue",3],["Angular",3],["Next.js",2],["Nuxt",2], …]}
```

`allFrameworks`
([FrameworkIdeMappings.svelte:152-160](../../../src/lib/domains/settings/components/FrameworkIdeMappings.svelte#L152))
concatenates `frameworkGroups` (the backend's suggested list) with `frameworks` (the seeded
`frameworks` table) without deduplicating by name, and `unmappedByCategory` then buckets the
merged list. The empty-mappings branch has the same problem a different way: it renders
`frameworksByCategory` and then `frameworkGroups` as two separate lists of the same names.

Pre-existing (`getAllFrameworks()` on `main` concatenates the same two sources), carried through
the split unchanged.

## Evidence

`.claude/qa/shots/ides-mappings-duplicate-chips.png`

## Repro

`node .claude/qa/scripts/ides-verify6.mjs`

## Fix

Root cause: `allFrameworks` concatenated `frameworkGroups` (suggested) and `frameworks`
(database) without deduplicating, and the no-mappings branch additionally rendered both as two
separate lists. Replaced with `dedupedFrameworks` — a single pass over `frameworks` then
`frameworkGroups`, keyed by lowercased name, database entries win a collision (so the chip
stays editable) — then both branches render from `chipsByCategory`/`unmappedChipsByCategory`
built on top of it. Each framework now appears exactly once everywhere.

File: `src/lib/domains/settings/components/FrameworkIdeMappings.svelte` (same edit as
QA-096/QA-102/QA-095 — one file, one rendering pass).
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-103.mjs` — counted chip names in both the
no-mappings branch and the has-mappings branch (after creating a fixture IDE + mapping): 53
distinct framework names, 0 duplicates in either branch (the one "Toggle Sidebar" dupe in the
raw count is unrelated page chrome caught by the broad `button` selector, not a framework
chip). `.claude/qa/shots/QA-103-no-mappings-branch.png`,
`.claude/qa/shots/QA-103-has-mappings-branch.png`. Fixtures cleaned up. Verified.
