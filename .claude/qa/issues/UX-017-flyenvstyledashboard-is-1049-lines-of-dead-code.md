---
id: UX-017
title: FlyEnvStyleDashboard is 1049 lines nothing renders
status: fixed
severity: minor
kind: ux
area: sdk
mode: code-read
found: 2026-11-06
---

## Finding

`src/lib/domains/sdk/components/FlyEnvStyleDashboard.svelte` is **1049 lines** — the third
largest component in the app. The only reference to it anywhere in `src/` is the barrel
re-export:

```
src/lib/domains/sdk/index.ts:15
export { default as FlyEnvStyleDashboard } from './components/FlyEnvStyleDashboard.svelte';
```

No route, layout or component imports it. A search for `FlyEnvStyleDashboard` across
`src/**/*.{svelte,ts}` returns the component itself, two of its own `logger` context
strings, and that export line. It never renders.

It carries its own copy of several problems the rest of the sweep is fixing: a hand-rolled
`text-3xl` header (UX-003), a `md:grid-cols-4` stat grid (UX-002), a
`TabsList grid w-full grid-cols-3` (UX-004), 21 Dialog usages, and a card titled
"FlyEnv-Style Features" that advertises the app to itself. Every future codemod over those
patterns pays for it.

Note `AliasManager.svelte` and `CustomDirectoryManager.svelte` only *mention* FlyEnv in a
comment — they are live and are not part of this.

## Fix

Delete `src/lib/domains/sdk/components/FlyEnvStyleDashboard.svelte` and the export at
`src/lib/domains/sdk/index.ts:15`. Then `pnpm check && pnpm lint`.

If the owner wants it kept as a reference, that is a `needs-decision` — but it has no
caller, so nothing regresses either way.

## Evidence

`rg "FlyEnvStyleDashboard" src -g "*.svelte" -g "*.ts"` → 1 export + self-references only.

## Repro

Code read.

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->
