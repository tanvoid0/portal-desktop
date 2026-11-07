---
id: QA-102
title: Framework chip's invisible hover overlay swallows the click — the centre of "React" opens Edit Framework
status: open
severity: major
kind: bug
area: settings/ides
mode: tauri
found: 2026-11-07
---

## Use case

On `/settings/ides`, click a framework chip to quick-add a mapping for it.

## Steps

1. `/settings/ides` with no mappings yet.
2. Click the middle of the **React** chip under FRONTEND.

## Expected

"Add Framework IDE Mapping" opens, prefilled with React.

## Actual

**"Edit Framework"** opens instead. The chip's edit/delete overlay
([FrameworkIdeMappings.svelte:265](../../../src/lib/domains/settings/components/FrameworkIdeMappings.svelte#L265))
is `absolute top-1 right-1 … opacity-0 group-hover:opacity-100` — `opacity: 0` with
`pointer-events: auto`, so it is invisible but clickable at all times, and it is 52px wide on an
89px chip:

```
geometry: {"chip":{"x":616,"w":89},"overlay":{"x":649,"w":52},"opacity":"0","pe":"auto"}
elementFromPoint(chip centre) -> BUTTON | (no label) | INSIDE HOVER OVERLAY
click chip centre    -> "Edit Framework | Framework Name | Category | Icon …"
click chip left edge -> "Add Framework IDE Mapping | Framework …"   (the intended action)
```

The right 24px of that overlay is the **delete** button, so the right-hand quarter of every
custom framework chip is an invisible "delete this framework" target. Only the leftmost ~33px of
the chip does what the chip says it does.

The same overlay is the only way to edit or delete a custom framework, so these two invisible
buttons are both the hazard and the feature.

Pre-existing, not introduced by the IdeSettings split — `main`'s `IdeSettings.svelte` has the same
block at lines 760-790 — but it is still live after the refactor.

## Evidence

`.claude/qa/shots/ides-chip-centre-click.png`, output above.

## Repro

`node .claude/qa/scripts/ides-chip-click.mjs`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->
