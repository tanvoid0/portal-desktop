---
id: UX-013
title: Ten cloud list routes are the same 200-line file with a different ResourceType
status: verified
severity: major
kind: ux
area: cloud
mode: tauri (cluster not connected — code read + guard screen)
found: 2026-11-06
---

## What I could and could not test

No Kubernetes cluster is configured on this machine, so every `/cloud/*` route renders
`CloudConnectionGuard` ([src/routes/cloud/+layout.svelte:143](../../../src/routes/cloud/+layout.svelte#L143))
— `main.innerText.length` is 31 on `/cloud/workloads` and `/cloud/workloads/pods`. The
findings below are from reading the ten files, which are near-identical; the density numbers
come from the markup, not from a running cluster. **The IA recommendation does not depend on
the cluster; the pixel claims should be re-measured once one is connected.**

## Problem

| route | lines | PageHeader | stat Cards |
| --- | --- | --- | --- |
| cloud/workloads/pods | 152 | no | 0 |
| cloud/workloads/services | 177 | no | 4 |
| cloud/workloads/deployments | 384 | no | 4 |
| cloud/workloads/jobs | 205 | no | 5 |
| cloud/workloads/cronjobs | 208 | no | 5 |
| cloud/workloads/daemonsets | 196 | no | 4 |
| cloud/workloads/statefulsets | 203 | no | 4 |
| cloud/configmaps | 181 | no | 3 |
| cloud/secrets | 204 | no | 3 |
| cloud/ingress | 167 | no | 3 |

`diff cloud/workloads/daemonsets/+page.svelte cloud/workloads/statefulsets/+page.svelte`
produces **87 changed lines out of ~200** — and all 87 are the words "DaemonSet" /
"StatefulSet", the `ResourceType` constant, and the column array. Everything else is
character-for-character the same: the same `onMount` + `loadResources`, the same
`filtered*` derived, the same `*Stats` derived with total/running/pending/failed, the same
`statusOptions` derived, the same `useTableNavigation` + `useResourceActions` wiring with
the same four handlers, the same `text-3xl` header, the same four stat Cards, the same
search + status Select row, the same `BaseResourceTable`.

Each one therefore carries its own copy of UX-002 (four stat Cards) and UX-003
(`text-3xl` header) — ten copies of two problems that are each one fix in one file.

## Fix — collapse into one resource browser

**Route map before:**

```
/cloud/workloads
/cloud/workloads/{pods,services,deployments,jobs,cronjobs,daemonsets,statefulsets}
/cloud/workloads/{...}/[name]
/cloud/{configmaps,secrets,ingress}
/cloud/{configmaps,secrets,ingress}/[name]
/cloud/{configmaps,secrets}/new
```
20 route files.

**Route map after:**

```
/cloud/workloads                 unchanged — the cluster overview
/cloud/[resource]                one list page, ResourceType from the param
/cloud/[resource]/[name]         one detail page
/cloud/[resource]/new            one create page (only configmaps + secrets enable it)
```
4 route files.

`CloudNavigation` already owns the resource list, so it is the natural home for the
`ResourceType` → `{ label, columns, canCreate, detailTabs }` table. Put that table in
`src/lib/domains/cloud/core/resourceDescriptors.ts`; `/cloud/[resource]/+page.svelte`
becomes ~60 lines: look the descriptor up, `loadResources(descriptor.type)`, render
`PageHeader` + `PageStats` + `BaseResourceTable`.

Redirect the old paths (`/cloud/workloads/pods` → `/cloud/pods`) in
`src/routes/cloud/workloads/[...rest]/+page.ts` so bookmarks and the command palette
(`src/routes/cloud/+layout.svelte:57-90`, which hardcodes four `goto` targets) keep working;
then update the palette to generate its entries from the same descriptor table instead of
listing four of the ten by hand.

The seven detail pages (`[pod]` 910 lines down to `[statefulset]` 230) are a second,
larger pass — do the list pages first and judge the detail ones separately, since
`pods/[pod]` genuinely has more to show (logs, exec, containers) than `daemonsets/[name]`.

## Evidence

`.claude/qa/shots/ux/ux-cloud-workloads.png` (connection guard — all that renders here).
`diff src/routes/cloud/workloads/daemonsets/+page.svelte src/routes/cloud/workloads/statefulsets/+page.svelte`

## Repro

Code read — `/cloud/*` cannot be exercised without a cluster.

## Fix

Root cause: ten list routes each hardcoded one `ResourceType` instead of reading a
descriptor, so every UX problem on them existed in ten copies.

Added `src/lib/domains/cloud/core/resourceDescriptors.ts` (10 rows: slug, ResourceType,
label, icon, group, keywords, columns, stats, filter, detailBase, canCreate/hasLogs/canScale)
plus `src/routes/cloud/[resource]/{+page.svelte,+page.ts}` (234 + 11 lines) and
`src/routes/cloud/workloads/[...rest]/+page.ts` (307 redirects for the seven old workload
list paths; configmaps/secrets/ingress keep their URLs and are now served by `[resource]`).
Deleted the ten list pages, four pods-only components and the unreferenced `WorkloadTabs`.
Repointed the workloads overview tiles and the seven detail-page back links; the sidebar,
its digit shortcuts and the command palette (previously 4 of 10 hardcoded) all generate from
the descriptor table. `CloudNavigation` went 250 -> 159 lines.

25 route files -> 18. The ten list pages were 1875 lines; the replacement is 579 including
the shared table.

Detail and `new` routes are untouched and were deliberately NOT swallowed by the redirect —
verified with SvelteKit's own `create_manifest_data` + `parse_route_id`/`exec` over the real
route tree across 29 URLs, and independently re-checked against the route directories: the
only `/cloud/workloads/<plural>` strings left in `src/` are the seven `detailBase` values and
two detail pages' own tab-switch `goto`s, which are correct because those detail routes do
still live there.

Also covers UX-002 and UX-003 for these ten routes (PageStats strip and PageHeader in place
of hand-rolled stat Cards and `text-3xl` headers).

Deliberate losses, reviewed: pods' `KeyboardShortcutsPanel` (the layout's `?` help covers
it) and secrets' Show/Hide toggle (dead UI — `showSecrets` never reached the table). The
deployments per-row Scale button was initially dropped and has been RESTORED as an opt-in
`rowActions?: Snippet<[ICloudResource]>` on `BaseResourceTable`, hover-revealed via
`opacity-0 group-hover:opacity-100 focus-within:opacity-100` so it stays in the tab order,
with `aria-label="Scale <name>"`. Only `canScale` resources render the column.

A pre-existing bug found and fixed in passing is filed separately as QA-089
(`useResourceActions` read its list getter once at subscribe, so the action shortcuts
operated on the empty pre-load list on all ten old pages).

Checks: `pnpm check` 0 errors (3814 files), `npx eslint src` 0 errors, prettier and
`scripts/check-shadcn-ui.mjs` clean. No Kubernetes cluster on this machine, so every
`/cloud/*` route renders `CloudConnectionGuard` and none of this was exercised against a
live cluster — verification is by code read and route-matcher simulation only.

## Verification

**2026-11-07 — verified by code read and route-matcher simulation. Nothing was clicked.**

There is still no Kubernetes cluster on this machine, and `src/routes/cloud/+layout.svelte:108`
only renders `{@render children()}` inside `{#if $cloudStore.connection.isConnected}` — so
not one of these eleven pages can be mounted here at all. The dev server was deliberately
not started: it would have shown the same `CloudConnectionGuard` the original evidence shot
already shows, and a second Portal instance (`:1421` / CDP `:9223`) was in use by another
agent. **No `QA-…-after` screenshot exists for this issue and none can until a cluster is
connected.** Everything below is a code read plus a route resolution run against the real
route tree.

Verified on commit `d8ce6e3` (`refactor(ui): compact page chrome and collapse duplicate
routes`), baseline `d0fccf8`.

### What I ran

`node .claude/qa/scripts/UX-013-routes.mjs` — builds the manifest from `src/routes/` with
SvelteKit 3.1.0's own `create_manifest_data`, then resolves 33 URLs through each route's
`pattern` + `exec` in manifest order, which is the order the client router walks. (There is
no `svelte.config.js` in this repo; the kit options live inline in `vite.config.ts` and set
nothing that affects routing, so plain defaults are faithful.) **33/33 URLs, 10/10
`detailBase` values and 11/11 sidebar active-state cases pass.**

Also run: `npx svelte-check --tsconfig ./tsconfig.json` → **0 errors**, 133 pre-existing
warnings. `npx eslint src/routes/cloud src/lib/domains/cloud` → **0 errors**, 90
pre-existing warnings (so `no-derived-closure`, `no-legacy-syntax` and `no-raw-ui-elements`
are all clean on the new code). `npx prettier --check` on the five new/rewritten files →
clean. `svelte-kit sync` was not re-run; `.svelte-kit/types/src/routes/cloud/` already
contains `[resource]/` and `workloads/[...rest]/`, so the type gate did see the new routes.

### 1. Redirects — pass

| URL | route that matches | why | lands on |
| --- | --- | --- | --- |
| `/cloud/workloads` | `/cloud/workloads` | a static segment outranks a rest param, so `[...rest]` (which can match zero segments) never sees it | overview, unchanged |
| `/cloud/workloads/pods` | `/cloud/workloads/[...rest]` | no `/cloud/workloads/pods/+page` exists any more; `pods/[pod]` needs two segments | 307 → `/cloud/pods` |
| `…/services` · `…/deployments` · `…/jobs` · `…/cronjobs` · `…/daemonsets` · `…/statefulsets` | `/cloud/workloads/[...rest]` | same | 307 → `/cloud/<slug>` |
| `/cloud/workloads/nonsense` | `/cloud/workloads/[...rest]` | `getResourceDescriptor` returns undefined | 307 → `/cloud/workloads` |
| `/cloud/workloads/pods/extra/deep` | `/cloud/workloads/[...rest]` | three segments, no page that deep; `rest` = `"pods/extra/deep"`, no descriptor | 307 → `/cloud/workloads` |
| `/cloud/configmaps` · `/cloud/secrets` · `/cloud/ingress` | `/cloud/[resource]` | their own `+page.svelte` was deleted, so the dynamic sibling takes the single segment. No redirect, URLs unchanged | list page |
| `/cloud/junk` | `/cloud/[resource]` | matches the pattern, then `+page.ts` calls `error(404, …)` | root `+error.svelte` renders "404 Page not found — There is nothing at /cloud/junk" with Go back / Go home. Not a blank page |
| `/cloud` | `/cloud/+page.ts` | unchanged | 307 → `/cloud/workloads` |

Query strings survive: `redirect(307, \`/cloud/${descriptor.slug}${url.search}\`)`
(`workloads/[...rest]/+page.ts:21`). The junk fallback drops the search string, which is
right — the target is a different page. 307 (not 301) means the browser never bakes the
redirect into a bookmark.

`/cloud/workloads/[...rest]/` has a `+page.ts` and no `+page.svelte`. That is safe here for
two reasons: `load` throws on **both** branches, so the (absent) component is never
rendered; and the identical pattern already ships in this app at `src/routes/cloud/+page.ts`.

### 2. No detail URL is swallowed — pass

All twelve detail/create URLs reach their own page. The rule in each case is SvelteKit's
route specificity sort (`create_manifest_data/sort.js`), applied segment by segment:
**static beats dynamic beats rest**, and a route is only considered if its segment count
fits.

- The seven `/cloud/workloads/<plural>/<name>` URLs resolve to
  `/cloud/workloads/<plural>/[<singular>]`: segment 2 is the static `<plural>`, which
  outranks `[...rest]`'s greedy match of the same two segments. Confirmed in the manifest
  order printed by the script — all seven detail routes sort **above** `/cloud/workloads/[...rest]`.
- `/cloud/configmaps/my-cm`, `/cloud/secrets/my-secret`, `/cloud/ingress/my-ing` resolve to
  their own `[configmap]` / `[secret]` / `[ingress]` routes: two segments, and
  `/cloud/[resource]` is one segment, so it cannot match at all.
- `/cloud/configmaps/new` and `/cloud/secrets/new` resolve to the static `new` routes,
  which sort above the sibling `[configmap]` / `[secret]`.
- `/cloud/pods/new` correctly 404s — pods has no `new` route and `canCreate` is false, so
  nothing links there.

### 3. `detailBase` — pass, 10/10

Each value checked against the directory that actually exists, by resolving
`<detailBase>/sample-name` and asserting it lands on a `[param]` route:

| slug | `detailBase` | real route |
| --- | --- | --- |
| pods | `/cloud/workloads/pods` | `/cloud/workloads/pods/[pod]` |
| services | `/cloud/workloads/services` | `/cloud/workloads/services/[service]` |
| deployments | `/cloud/workloads/deployments` | `/cloud/workloads/deployments/[deployment]` |
| statefulsets | `/cloud/workloads/statefulsets` | `/cloud/workloads/statefulsets/[statefulset]` |
| daemonsets | `/cloud/workloads/daemonsets` | `/cloud/workloads/daemonsets/[daemonset]` |
| jobs | `/cloud/workloads/jobs` | `/cloud/workloads/jobs/[job]` |
| cronjobs | `/cloud/workloads/cronjobs` | `/cloud/workloads/cronjobs/[cronjob]` |
| configmaps | `/cloud/configmaps` | `/cloud/configmaps/[configmap]` |
| secrets | `/cloud/secrets` | `/cloud/secrets/[secret]` |
| ingress | `/cloud/ingress` | `/cloud/ingress/[ingress]` |

### 4. No stale links — pass

`grep -rn '/cloud/workloads/(pods|services|deployments|jobs|cronjobs|daemonsets|statefulsets)' src/`
returns ten hits: seven `detailBase` values, one comment in `[...rest]/+page.ts`, and the
daemonset and statefulset detail pages' own tab-switch `goto`s
(`[daemonset]/+page.svelte:120`, `[statefulset]/+page.svelte:127` — correct, those detail
routes do still live there). All seven workload detail pages and the three configmap /
secret / ingress detail and `new` pages now back-link to `/cloud/<slug>`.

### 5. Deleted components — pass

Nothing in `src/` references `WorkloadTabs`, `PodsTable`, `PodsStatistics`, `PodsFilters`,
`WorkloadListShell` or `components/workloads`. `BaseResourceTable` is now imported by
exactly one file (`src/routes/cloud/[resource]/+page.svelte`). The only surviving mentions
are stale rows in `docs/development/SHADCN_COMPONENT_AUDIT.md:143`,
`docs/development/SVELTE_PATTERNS_REVIEW.md:26-29,302` and
`docs/status/PROGRESS_REPORT.md:30,54` — doc drift, not wiring.

### 6. Palette and sidebar — pass

`+layout.svelte:67-76` is a `RESOURCE_DESCRIPTORS.forEach` producing one `nav-<slug>`
command each, so all ten are in the palette (was four of ten hardcoded) with `keywords`
from the descriptor. `CloudNavigation.svelte` builds its three groups from
`GROUP_ORDER.map(... RESOURCE_DESCRIPTORS.filter(group === title))` — no hardcoded item
list survives, 250 → 159 lines.

Digit shortcuts unchanged: `RESOURCE_TYPE_SHORTCUTS[slug.toUpperCase()]` resolves for nine
slugs (`1`–`9`) plus `0` for Overview; `ingress` has no entry and the `flatMap` drops it —
exactly the set the old hand-written list had, so no regression. (Ingress having no digit
is a pre-existing gap in `keyboardConstants.ts`, not something this change introduced.)

Active state, replicating `isActiveTab` over the real descriptor table: for each of the ten
resources, both `/cloud/<slug>` and `<detailBase>/<name>` highlight exactly that one item
and nothing else; `/cloud/workloads` highlights only Overview. `/cloud/configmaps/new` and
`/cloud/secrets/new` highlight their resource (the `/cloud/<slug>/` prefix covers them). No
slug is a prefix of another and every prefix ends in `/`, so no cross-highlighting is
possible. This is a strict improvement — the old version needed four hand-written special
cases.

### 7. Scale snippet — pass

`BaseResourceTable.svelte:264-270` renders the cell unconditionally when `rowActions` is
given and only animates `opacity-0 → group-hover:opacity-100 focus-within:opacity-100` —
it is in the DOM, in flow and in the tab order, not `hidden` and not `display:none`. The
button carries `aria-label="Scale {resource.name}"` and its `onclick` calls
`event.stopPropagation()` before `openScaleDialog`, so it cannot also fire the row's
`onclick` → `openDetail`
(`src/routes/cloud/[resource]/+page.svelte:187-199`).

With `rowActions` undefined the table is byte-for-byte the old one: no extra `TableHead`,
no extra `TableCell`, and the empty-state `colspan={defaultColumns.length + (rowActions ? 1 : 0)}`
evaluates to the old `defaultColumns.length`. Only `deployments` sets `canScale`, and its
descriptor has no `actions` column, so there is never a duplicate "Actions" header.
`BaseResourceTable` has exactly one consumer, so the new prop cannot affect anything else.

### 8. Conventions — pass

`$props()` + `interface Props`, `$state` / `$derived` / `$derived.by` only (no
`$derived(() => …)`), snippets for `actions` / `filters` / `rowActions`, `onclick` not
`on:click`, no `export let`, no `<slot>`. Controls are `Button` / `Input` / `Label` /
`Select` / `Dialog` from `$lib/components/ui/*`, page chrome is `PageHeader` / `PageStats` /
`PageFilters` / `PageLoading` from `$lib/components/shell`, toasts via `$lib/utils/toast`.
All imports use `$lib/`. `git diff d0fccf8 HEAD --name-only | grep components/ui/` returns
nothing — the vendored primitives were not touched. eslint confirms the three `portal/*`
rules are clean.

### Also confirmed

QA-089's fix is present at `src/routes/cloud/[resource]/+page.svelte:127` —
`resources: toStore(() => filtered)` rather than the bare getter. `useResourceActions`'s
getter branch (`useResourceActions.ts:46-52`) still calls `fn()` once at subscribe and
returns a no-op unsubscribe, so the trap is intact in the hook; the list page is now its
only caller, so nothing is broken today. See QA-089.

### Filed against this change

- **QA-093** (minor) — the deployments list lost its `onRestart` handler. Because
  `ACTION_SHORTCUTS.RESTART` and `REFRESH` are both `'r'`, dropping it silently rebinds `r`
  from Restart to Refresh, and `ShortcutsHelp` still advertises `r → Restart`. Not in the
  "deliberate losses, reviewed" list above. Mitigated: restart still exists on the
  deployment detail page, and QA-089 means the list shortcut never worked for a user anyway.
- **QA-092** (minor) — the cluster overview's Quick Actions grid is still ten hand-written
  buttons. It has no Ingress tile and one tile that navigates to the page it is on. The
  `goto` targets were correctly repointed by this change, but the list itself is the one
  place the UX-013 root cause survives. Pre-existing drift, not a regression.

Neither blocks this issue: the ten routes are collapsed, the URLs all resolve, and the
sidebar and palette are generated. **status: verified.**

### Still unproven until a cluster exists

Nothing below the connection guard has ever been rendered. Everything that needs real data
is untested: the list actually populating, `PageStats` numbers, the status/CronJob filters,
the search box, `useTableNavigation` arrow selection and the `k8s-navigable-table`
highlight, the `d`/`e`/`y`/`l`/`f`/`s` shortcuts end to end, the Scale dialog and
`scaleDeployment` round-trip, the hover reveal and tab order of the Scale button with real
rows, light/dark and 1024×600 layout of the new page, long resource names and many rows,
the `$effect` re-load when you switch slugs in the sidebar, and whether a failed
`loadResources` shows anything other than the "No X found" empty state (neither the old
pages nor the new one renders `PageError`).
