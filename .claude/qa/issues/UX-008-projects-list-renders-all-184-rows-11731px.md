---
id: UX-008
title: Projects renders all 184 rows and 33 group headers at once — 11731px of DOM, no way to collapse
status: fixed
severity: major
kind: ux
area: projects
mode: tauri
found: 2026-11-06
---

## Use case

Find one project among 184.

## Measured (1280x800, real data)

| thing | measured |
| --- | --- |
| rows rendered | **184**, all at once |
| groups | **33**, of which **11 contain a single project** |
| `<button>` elements inside `main` | **190** |
| scroll height | **11 731px** in a 760px viewport — 15.4 screens |
| row height | 48px |
| group header height | 27px |

Nothing collapses, nothing virtualises, and `ProjectRow` renders a `…` menu button per row,
so a single Tab sweep of the page crosses 190 controls. The 11 single-project groups each
cost a 27px header plus a divider to hold one row.

The row itself leaves its middle third empty: name on the left, stack badges starting around
x=630, activity around x=940. At 1024px wide (sidebar open) the badges and the date collide
into the name's truncation instead.

## Expected

A list this long either virtualises or collapses. Either way a developer should be able to
get to a project without 15 screens of scrolling.

## Actual

184 rows, always expanded, always in the DOM.

## Fix

In [src/routes/projects/+page.svelte:302-331](../../../src/routes/projects/+page.svelte#L302):

1. Wrap each `<section>` body in the existing `Collapsible` primitive
   (`$lib/components/ui/collapsible`), group header as the trigger with the count badge it
   already shows. Persist open/closed per group key in `localStorage`. Default: open only
   the groups that contain a project touched in the last 30 days — the "Recently active"
   sort already knows this.
2. Groups of one: render the row directly with no header, prefixing the row's name with the
   group path in `text-muted-foreground`. Removes 11 headers and 11 dividers.
3. Row: drop the per-row `…` menu from the default render; show it on `group-hover` only
   (the row is already `class="group"`). That takes the tab-stop count from 190 to ~10.

If the owner wants everything expanded, the virtualisation alternative is
`@tanstack/svelte-virtual` over the flattened `groups` array — but collapsing is the smaller
diff and matches how the data is already shaped.

## Evidence

`.claude/qa/shots/ux/ux-projects-2.png`, `.claude/qa/shots/ux/ux-min1024-projects.png`.

## Repro

`node .claude/qa/scripts/app.mjs projects ux-projects` — then in devtools:
`document.querySelector('main .overflow-y-auto').scrollHeight` → 11731.

## Fix

<!-- coder -->

Root cause: every area section rendered its rows unconditionally, so the whole 184-row list was
always laid out — nothing collapsed and single-project areas each paid a header plus a divider.

Changes:

- `src/routes/projects/+page.svelte` — each multi-project section is now wrapped in the existing
  `Collapsible` / `CollapsibleTrigger` / `CollapsibleContent` (`$lib/components/ui/collapsible`);
  the `<h2>` is the trigger and keeps its count badge, with a chevron rotating off
  `group-data-[state=open]`. Open/closed persists per group key under
  `portal.projects.openGroups` in `localStorage` (read and write both in try/catch, plus a
  `typeof localStorage === 'undefined'` guard — a throw, missing key or junk value falls back to
  the defaults and the page renders unchanged). Default open = groups holding a project whose
  `lastSeen()` (the same `last_activity ?? last_opened` the "Recently active" sort uses) is within
  30 days. While any search/filter is active every group is forced open and the trigger is
  `disabled`, so filtering still shows all matches. Areas holding exactly one project with no
  nested children (and not the Starred section) render the row directly with no header — the row
  shows the area path in `text-muted-foreground` before the name. Rows now go through one
  `{#snippet row(project, nested, pathPrefix)}` instead of three duplicated `ProjectRow` blocks.
- `src/lib/domains/projects/components/ProjectRow.svelte` — new `pathPrefix` prop; the `…`
  dropdown trigger is `opacity-0 transition-opacity group-hover:opacity-100
  focus-within:opacity-100 data-[state=open]:opacity-100`, so it stays in the tab order and in the
  accessibility tree and becomes visible on keyboard focus or while its menu is open (no `hidden`,
  no `display:none`). Layout: the name block dropped `flex-1` for `shrink`, the stack column lost
  its fixed `w-72` for `max-w-56 shrink-0` so badges sit beside the name, the activity column got
  `ml-auto` to pin the right-hand columns, and the git column moved from `lg:` to `xl:` at `w-36`.
  At 1024 with the sidebar open that leaves the name ~340px instead of ~110px, and the row height
  is unchanged (48px, single line).

Scroll-height arithmetic (760px viewport): the all-collapsed floor is 11 lone rows (528px) + 22
headers (682px) + section gaps (768px) ≈ 1 980px, 2.6 screens against the measured 11 731px /
15.4 screens; with the 30-day default opening k groups add 48px per row rendered. Buttons inside
`main` drop from 190 to 6 page controls + 22 triggers + one per rendered row (39 all-collapsed).
Note the ticket's "~10 tab stops" is not reachable without removing the menus from the tab order,
which would break keyboard access — at rest zero row menus are *visible*, but the ones in open
groups stay focusable by design.

Checks: `npx prettier --check` clean, `npx eslint src/routes/projects src/lib/domains/projects`
0 errors (77 pre-existing warnings), `pnpm check` 0 errors with no warning in either touched file.
The repro script could not be run — the app was not listening on CDP :9222 and this session was
told not to start the dev server, so the before/after numbers above are arithmetic, not measured.

## Verification

<!-- qa-tester -->
