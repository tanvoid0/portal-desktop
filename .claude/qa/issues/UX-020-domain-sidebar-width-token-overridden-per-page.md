---
id: UX-020
title: --sidebar-width-domain is 20rem for one-line nav items, so pages now override it inline one at a time
status: fixed
severity: minor
kind: ux
area: all (domain sidebars)
mode: tauri
found: 2026-11-07
---

## How this surfaced

Not from a page sweep — from two independent fixes in the UX-005/UX-012 wave both reaching
for the same workaround.

`app.css` sets `--sidebar-width-domain` to **20rem (320px)** at `:root` and **14rem (224px)**
below `80rem`. `ShellSidebarLayout` reads `--sidebar-width: var(--sidebar-width-domain)`, so
every domain sidebar — settings, sdk, cloud, tasks' project rail — is 320px on a wide window.

That width was sized for **two-line nav items**. UX-005 deleted the description line from the
settings nav (items are now one line, `h-8`), and the widest remaining label,
"Frameworks & Languages", needs far less than the 288px the card gets. The same is true of the
SDK sidebar after UX-018 collapsed its two trees, and of the tasks project rail, which holds
five entries and a count.

Two fixes in the same wave then diverged:

| page | what it does now | effective rail |
| --- | --- | --- |
| `/settings` | uses the token as-is; the Card is `w-full` | 224px ≤80rem, **320px** above |
| `/tasks` | pins `--sidebar-width-domain: 14rem` inline on `TaskManager`'s root | **224px** at every width |
| `/sdk`, `/cloud` | untouched | 224px ≤80rem, **320px** above |

An earlier attempt on `/settings` pinned the token to `16rem` inline and was rejected,
because an inline value *also* overrides the narrow-screen rule — it made the rail **wider**
than 14rem on exactly the small windows this whole sweep is about. `/tasks`' 14rem happens to
be safe only because it matches the narrow value by coincidence.

## Problem

The token is the wrong width for the content it now holds, so each page is settling it
privately with an inline style. Three consequences:

1. A per-page inline `--sidebar-width-domain` silently overrides the breakpoint rule as well
   as the base value. Any future value that is not 14rem regresses small screens, and nothing
   in the CSS makes that visible at the call site.
2. Two domain sidebars are now different widths at the same window size for no reason a user
   could name.
3. 96px of a 1232px content area, on every wide-window settings, sdk and cloud page, holds
   nothing.

## Expected

One width for domain sidebars, set once where the token is defined, correct for one-line
items — and no page-level inline override.

## Fix

Lower `--sidebar-width-domain` at `:root` in `src/app.css` from `20rem` to about `14rem`,
check the four domain sidebars at 1280 and above, then delete the inline override on
`TaskManager`'s root. Check `ShellSidebarLayout`'s hardcoded `20rem` fallback at the same
time. If one sidebar genuinely needs more room than the others, that is an argument for a
second named token, not for an inline style on a component.

Do this as one deliberate pass with the four sidebars open side by side — it is a shared
token, so it is the one change here that should not be made a page at a time.

## Evidence

Code read. `src/app.css` (`--sidebar-width-domain`),
`src/lib/components/shell/shell-sidebar-layout.svelte` (the `20rem` fallback),
`src/lib/domains/tasks/components/TaskManager.svelte` (the inline override),
`src/routes/settings/+layout.svelte` (uses the token as-is after UX-005).

## Repro

Open `/settings/general` and `/tasks` in a window wider than 1280px and compare the two left
rails.

## Fix

<!-- coder -->

Lowered `--sidebar-width-domain` to `14rem` at `:root` in `src/app.css` and deleted the
`width < 80rem` media query that duplicated it (base and narrow value were now the same).
Updated `ShellSidebarLayout`'s hardcoded fallback from `20rem` to `14rem`
(`shell-sidebar-layout.svelte:88`). Deleted the inline `--sidebar-width-domain: 14rem;`
override (and its now-stale comment) on `TaskManager`'s root — the shell default matches it
already. `pnpm check` and `pnpm lint` pass with no new warnings.

## Verification

<!-- qa-tester -->
