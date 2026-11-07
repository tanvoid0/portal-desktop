---
id: QA-103
title: '"Add More Frameworks" lists 70 chips for 30 frameworks — every name repeated two or three times'
status: open
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

<!-- coder -->

## Verification

<!-- qa-tester -->
