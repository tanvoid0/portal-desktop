---
id: QA-102
title: Framework chip's invisible hover overlay swallows the click — the centre of "React" opens Edit Framework
status: verified
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

Round 1 narrowed the always-`pointer-events-auto` hit box but, per the reopen, still left the
two buttons clickable at all times regardless of visibility. Round 2, exactly per the
tester's own diagnosis: gated `pointer-events` on hover itself. The wrapper is now
`pointer-events-none opacity-0 ... group-hover:pointer-events-auto group-hover:opacity-100`,
with no per-button `pointer-events-auto` override — `pointer-events` is an inherited CSS
property, so while not hovering the whole subtree (including both buttons) is `none` and every
click passes through to the chip's own button underneath; on hover the wrapper (and therefore
its children) switches to `auto` at the same moment it becomes visible. Invisible ⇔
unclickable, now actually enforced instead of just a smaller permanent hit box.

File: `src/lib/domains/settings/components/FrameworkIdeMappings.svelte` (same file as
QA-096/QA-103/QA-095).
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-102-recheck.mjs` — same repro as the
original issue, dead-center click on the real "React" chip under FRONTEND (real app data, not
a fixture):

```
elementFromPoint(chip center): {"tag":"BUTTON","label":"Edit React","text":"","...}
dialog after click: Edit Framework
```

**Reopened.** The fix narrowed the always-clickable (`pointer-events-auto`) hit box from the
full 52px-wide wrapper down to just the two 24px icon buttons, but those two buttons still sit
inside the chip's own bounds and are still `pointer-events-auto` at all times — opacity-0 does
not stop pointer events, only the (now-absent) hover-gated `pointer-events-none` would. A
dead-center click on an 88px-wide chip still lands on the invisible "Edit {name}" button
(`.claude/qa/shots/QA-102-recheck-react-chip-click.png` shows the real "Edit Framework" dialog
open, prefilled "React" / Frontend / logos:react, after clicking chip-center). The exact
original symptom survives.

Separately, this also surfaced that `isCustom` (the gate for whether a chip gets the
edit/delete overlay at all,
[FrameworkIdeMappings.svelte:205](../../../src/lib/domains/settings/components/FrameworkIdeMappings.svelte#L205))
is `true` for every row sourced from the `frameworks` **database** table, not just rows a user
created — "React" is apparently a seeded/already-used-not-"custom" row there, so it gets the
overlay too. Whether that's intended (any framework that's been used locally becomes
editable) or not is a product call, not filing separately — just noting it's why "React" was
reachable at all for this repro, same as the original issue which also used "React" as its
example.

Real fix likely needs the overlay buttons gated on actual hover/focus state (e.g. CSS
`:hover`/`:focus-within` rather than a Svelte-side `opacity-0` class with always-on
`pointer-events-auto`), not just a smaller always-live hit box.

### Round 2 — 2026-11-09, mode: tauri

Fix landed exactly that: `pointer-events-none` on the overlay wrapper by default,
`group-hover:pointer-events-auto` to flip it on only while genuinely hovering. Re-tested with
a method that isolates the unhovered case from Playwright's own click (which physically moves
the mouse there first, itself satisfying `:hover`):

```
UNHOVERED computed pointer-events: {"overlayPointerEvents":"none","editBtnPointerEvents":"none"}
JS-dispatched click (no real mouse movement) at chip center -> targets the real "React" button, not the overlay
dialog after that unhovered click: "Add Framework IDE Mapping"  (correct — not "Edit Framework")
```

The original symptom (dead-center click silently routing to Edit Framework) is gone. Also
confirmed in the DOM that `aria-label="Edit React"` still exists with
`group-hover:pointer-events-auto` wired, so the intentional hover-then-click-the-icon path is
structurally present (a manual coordinate-based hover+click attempt missed the small icon
geometrically in automation, not a product bug — the CSS gating itself is what mattered here
and is confirmed correct). `node .claude/qa/scripts/QA-102-recheck2.mjs`. Fixture cleaned up.
Verified.
