---
id: QA-109
title: Switch (toggle) unchecked track is nearly invisible on dark Card backgrounds — reads as a plain dot, not a toggle
status: verified
severity: minor
kind: ux
area: shared/theme (reproduced via devices)
mode: tauri
found: 2026-11-11
---

## Use case

Devices → "Pair a phone" → "Capabilities to grant" uses a `Switch` per
capability (via `CapabilityChecklist.svelte`); same for each pending
request's capability picker. All default to unchecked.

## Steps

1. Open Devices (`/devices`), dark theme (the app default).
2. Look at "Capabilities to grant" under "Pair a phone", or any pending
   request's capability list.

## Expected

Each capability shows a recognizable toggle (a pill-shaped track with a
thumb), per shadcn's `Switch`.

## Actual

Every unchecked switch renders as a small plain light dot with no visible
track/pill shape at all — indistinguishable from a static bullet until you
know to click it. Checked switches (e.g. the blue "Allow pairing over the
local network" toggle) are fine; only the unchecked state on a `Card`
background has this problem.

Measured computed colors (`devices` page, an unchecked switch inside a
`Card`):
- switch track: `oklab(0.280121 0.0000127852 0.00000560284 / 0.8)` (`bg-input/80`)
- enclosing card background: `rgb(27, 27, 27)` ≈ `#1b1b1b`
- page background: `rgb(18, 18, 18)`

Root cause: `src/app.css`'s dark theme sets `--input: var(--sidebar-border)`
(line ~154), and shadcn's vendored `Switch` (`src/lib/components/ui/switch/switch.svelte`)
uses `bg-input`/`dark:bg-input/80` for the unchecked track with
`border-transparent` — no border to fall back on. `--sidebar-border` is
tuned to be a subtle *border* color (1px line), not a *fill* a user has to
read as a discrete shape at a glance; against this app's near-black card
background the two are close enough in luminance that the track
disappears, leaving only the thumb visible.

This isn't specific to Devices — `Switch` is used in 17 places across the
app (grep `from '$lib/components/ui/switch'`) — but Devices' capability
checklists are the clearest repro since they default to several switches
off at once.

## Evidence

`.claude/qa/shots/sweep-devices-switch-zoom2.png` — three rows of
capability switches, each a plain dot with no visible track.
`.claude/qa/scripts/check-switch-contrast.mjs` output has the computed
colors above.

## Repro

`node .claude/qa/scripts/check-switch-contrast.mjs` (prints computed
colors) and `node .claude/qa/scripts/zoom-switch2.mjs` (screenshot).
Manual: open Devices in dark theme, look at any unchecked capability
switch.

## Fix

Root cause confirmed as described: dark theme's `--input` was aliased to
`--sidebar-border` (16% lightness), a value tuned for a 1px border line, not a
filled 0.8-alpha track a user has to read as a pill shape against the Card
background (10.5% lightness) — the two were too close in luminance (measured
contrast ~1.14:1).

Fix: decoupled `--input` from `--sidebar-border` in `.dark` and gave it its
own value, `0 0% 32%` (`src/app.css`, in the `.dark` block right after
`--border: var(--sidebar-border)`), with a comment explaining why. `--border`
stays aliased to `--sidebar-border` — unaffected.

No dedicated token was needed: `--input` backs two things app-wide —
`border-input` (hairline borders on Input/Textarea/Button/Radio/OTP/Calendar
selects) and the Switch's unchecked track fill. A single value can't be
"just right" for both; 32% is a middle ground that makes the switch track
clearly readable as a shape (contrast ~1.8:1 on Card, up from ~1.14:1) while
only modestly increasing border visibility elsewhere (contrast vs page bg
~1.3:1 → ~2.4:1) — a side effect worth flagging, not a regression: those
borders were arguably under-visible before too, and nothing exceeds the
`--ring`/`--primary` visual weight used for focus/active states. Didn't
split into a second CSS variable since the vendored `switch.svelte` can't be
edited to consume one (CLAUDE.md), so there was no way to target the fix
more narrowly without that edit.

Verified visually against the running app (Devices → capability checklist,
dark theme): unchecked switch track background went from
`oklab(0.280121 ... / 0.8)` to `oklab(0.43714 ... / 0.8)` against the same
`rgb(27, 27, 27)` Card — now reads as a pill, not a dot
(`.claude/qa/shots/sweep-devices-switch-zoom2.png`, recaptured after the fix
— note this filename is the same one QA's original evidence used; the
`zoom-switch2.mjs` script screenshots to a hardcoded path rather than
`app.shot()`'s no-overwrite helper, so the original before-fix PNG was
overwritten by this verification pass. The before-state numbers are still
recorded above in the issue's own Evidence section, so nothing load-bearing
was lost, but flagging it since "never overwrite a screenshot" is the
convention.)

This doesn't affect other QA ids (QA-106/107/108/110 untouched).

Checks: `pnpm check` — 0 errors (pre-existing unrelated warnings only).
`pnpm lint` — fails only on a pre-existing, already-committed formatting
issue in `.claude/handoff.md` unrelated to this change (confirmed via
`npx prettier --check src/app.css`, which passes clean).

Files changed: `src/app.css` only.

## Verification

2026-11-11, mode: tauri (real backend, CDP :9222, rebuilt from the working
tree including this fix). Re-measured the same unchecked switch on Devices:
track is now `oklab(0.43714 ... / 0.8)` (was `0.280121`), clearly a lighter
grey pill distinct from the `rgb(27,27,27)` card background. Visually
confirmed in `.claude/qa/scripts/zoom-switch2.mjs`'s screenshot and a
fresh clip (`.claude/qa/shots/QA-109-after-switch-track-visible.png`,
`.claude/qa/shots/QA-109-after-devices-full-page.png`) — every capability
switch in "Pair a phone" and the pending-request cards now shows a clear
pill-track + thumb, not a bare dot.

Sanity-checked other `--input`-dependent controls on the same pages for
regressions: `Input`/`Textarea` borders (Agents → Run form,
`.claude/qa/shots/qa-109-sanity-agents-run.png`) and the outline `Retry`
button (`QA-106-after-forced-error-3.png`) still look normal, not garish.
Did not spot any new contrast or border problems. Verified.

Note: my own `zoom-switch2.mjs` script writes via `page.screenshot()`
directly rather than `app.shot()`, so it overwrote
`sweep-devices-switch-zoom2.png` (the original before-fix close-up) with
the after-fix version instead of getting a `-2` suffix — the qualitative
"plain dot" description and computed colors in this issue's body are the
remaining before-fix record. Flagging so this doesn't look like a deleted
screenshot.

**Re-check 2026-11-11 (same day, separate pass):** cross-checked on
instance-2 (CDP :9223, sanctioned instance for this pass) via
`.claude/qa/shots/QA-116-reverify-baseline.png` — Devices page, dark theme,
"Capabilities to grant" row of unchecked switches and the checked "Allow
pairing over the local network" switch both read as clear pill+thumb
shapes, not dots. `src/app.css`'s `--input` value is unchanged since the
fix. Still verified.
