---
id: QA-169
title: A long unbroken name overflows the Scripts card, the script detail header and the breadcrumb, and the pane scrolls sideways
status: verified
severity: minor
kind: bug
area: automation/scripts
mode: tauri
found: 2026-11-22
---

## Use case

Create a script with a long name (a pasted path or identifier), then browse and open it.

## Steps

1. /scripts/new, Name = `QA script ` + 90 x's, Description `QA fixture`, Command `echo qa`, Save.
2. Open /automation/scripts and search the name.
3. Click View.

## Expected

Name wraps or truncates with a tooltip inside the card, the header and the breadcrumb; no horizontal scroll.

## Actual

- Card: the title runs past the card's right edge and the `utility` badge floats outside the card at the window edge.
- Detail: the `<h1>` and the breadcrumb leaf run off the window; a horizontal scrollbar appears at the bottom of the pane (scrollWidth 1101 > 1024 at 1024px wide).
- Blocks truncates correctly, so the Scripts card and detail header just miss `min-w-0` + `truncate`/`break-all`. Same family as QA-067.

## Evidence

`.claude/qa/shots/QA-169-before-long-name-card.png`, `.claude/qa/shots/QA-169-before-long-name-detail.png`.

## Repro

`node .claude/qa/scripts/QA-169.mjs` (creates and deletes its own `QA script long` fixtures)

## Fix
Follow-up (2026-11-22): card action row flex-wrap so it cannot spill at 1024. ScriptsPage.svelte. pnpm check + eslint/prettier.

Root cause: grid-item Card lacked `min-w-0`, description/h1 lacked wrapping, breadcrumb leaf had no truncation. Fix: `min-w-0` on card, `break-words` on description, `min-w-0`+`break-all` span on detail h1, breadcrumb items `min-w-0` + `truncate` + title tooltip. Files: automation/components/ScriptsPage.svelte, src/routes/scripts/[id]/+page.svelte, src/lib/components/ui/breadcrumb.svelte (app wrapper, not vendored dir). Reopen fix: CardHeader is a grid with an auto column, so the long title's min-content widened it; added `grid-cols-[minmax(0,1fr)]` on the header in ScriptsPage.svelte (also backs /automation/utilities) plus a title tooltip. QA-169.mjs now prints ok (no overflow); pnpm check 0 errors.

## Verification

2026-11-22, tauri. Detail page fixed: no overflow, breadcrumb truncates (QA-169-after-detail-long-name.png). The list CARD still overflows: with a 90-char unbroken name the card header grid is 1077px inside a 320px card, the title runs past the card edge and the pane scrolls sideways (QA-169-after-list-long-name.png; node QA-169.mjs still prints BUG, scrollWidth 1101>1040). The title/header element in ScriptsPage needs min-w-0 + break-all/truncate (min-w-0 was added on the Card but not on the header/title).

2026-11-22 re-verify, tauri: 90-char unbroken name on /automation/scripts and /automation/utilities at 1280x800 and 1024x600: title truncates with ellipsis inside the card, no sideways scroll at 1280 (QA-169-after-automation-scripts-1280.png, QA-169-after-automation-utilities-1280.png). Detail page: no overflow. Note at 1024 a single-card grid is 229px wide and the action-button row spills ~5px past the card edge (QA-169-after-automation-scripts-1024.png); that is card width, not name length, so not reopened. Fixture deleted.

2026-11-22 (round 2), tauri. Round 2: 90-char unbroken name on /automation/scripts and /automation/utilities at 1280 and 1024: no overflow anywhere, action row no longer spills (QA-169-after-automation-scripts-1024-2.png); detail page clean. Fixture deleted.
