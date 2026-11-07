---
id: UX-013
title: Ten cloud list routes are the same 200-line file with a different ResourceType
status: fixed
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

<!-- qa-tester -->
