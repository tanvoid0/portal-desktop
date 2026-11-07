---
id: UX-014
title: /sdk tabs render the same lists as /sdk/manager, /sdk/services and /sdk/versions — two implementations each
status: verified
severity: major
kind: ux
area: sdk
mode: tauri
found: 2026-11-06
---

## Use case

Install an SDK manager, or start an SDK service.

## Problem

`/sdk` is 867 lines with a four-tab control
([src/routes/sdk/+page.svelte:357-362](../../../src/routes/sdk/+page.svelte#L357)):

```
Overview | SDK Managers | Services | Versions
```

Each of the three non-overview tabs renders a list **inline**
(`TabsContent value="managers"` at `:792`, `"services"` at `:827`, `"versions"` at `:857`) —
and the same three lists also exist as standalone routes with their own, differently-styled
implementations:

| tab on /sdk | standalone route | lines |
| --- | --- | --- |
| SDK Managers (`:792-824`, card grid, "View Details" button) | [/sdk/manager](../../../src/routes/sdk/manager/+page.svelte) | 318 |
| Services (`:827-855`) | [/sdk/services](../../../src/routes/sdk/services/+page.svelte) | ~150 |
| Versions (`:857-...`) | [/sdk/versions](../../../src/routes/sdk/versions/+page.svelte) | 249 |

Both read the same TanStack queries (`sdkManagersQuery`, `sdkConfigsQuery`) and render
different UI from them. A fix to one is a fix to half the app's SDK surface.

There is also a **third** level of navigation to the same places: the 954-line
`SDKSidebar.svelte` in [src/routes/sdk/+layout.svelte](../../../src/routes/sdk/+layout.svelte)
occupies 355px on the left of every SDK page. On a cold load it showed one entry
("Overview") and "Loading SDKs…" twice — once in the body, once in the footer.

Net at 1280x800: 48px nav rail + 355px SDK sidebar + a page header + a stat grid + a
full-width tab bar, for a page whose useful content is a list.

## Expected

One place per thing.

## Fix

**Route map before:**
```
/sdk                       (tabs: overview, managers, services, versions — all inline)
/sdk/manager   /sdk/manager/[name]
/sdk/services
/sdk/versions
/sdk/software-installer
/sdk/[sdk]  /sdk/[sdk]/{models,package-manager,projects,service,version}
/sdk/ai/ollama
```

**After:**
```
/sdk                       overview only — stats strip + what is installed
/sdk/managers              (was /sdk/manager; plural, matches the sidebar label)
/sdk/managers/[name]
/sdk/services
/sdk/versions
/sdk/software-installer
/sdk/[sdk]  /sdk/[sdk]/{models,package-manager,projects,service,version}
/sdk/ai/ollama
```

1. Delete `TabsContent value="managers" | "services" | "versions"` and the `Tabs` wrapper
   from `/sdk/+page.svelte` (`:357` to the end), together with the `activeTab`
   state/`setActiveTab`. That is ~500 of its 867 lines.
2. Make the SDK sidebar the only navigation between those four views — it already lists
   them; it just had nothing to show while the query was pending.
3. Keep `/sdk/manager` as a redirect to `/sdk/managers` so existing links survive.
4. While there: `/sdk` and its siblings are 12 of the 19 files in UX-003 (hand-rolled
   `text-3xl` header) and `/sdk/+page.svelte:300-353` is one of the 13 in UX-002 (four stat
   Cards). Fixing this issue is the moment to do those two.

## Evidence

`.claude/qa/shots/ux/ux-sdk.png` — 355px sidebar showing only "Overview", four stat cards
reading `—`, four stretched tabs, and "Loading SDK data…" as the body.

## Repro

`node .claude/qa/scripts/app.mjs sdk ux-sdk`, then compare the "SDK Managers" tab with
`/sdk/manager`.

## Fix

<!-- coder -->

## Verification

2026-11-07 — tauri, dev instance 2 (vite :1421 / CDP :9223, own database).
Ran `node .claude/qa/scripts/ux014-verify.mjs`, `ux014-verify2.mjs`, `sdk-nav.mjs`.

Every redirect walked by hand:

| typed | landed on | h1 |
| --- | --- | --- |
| `/sdk/manager` | `/sdk/managers` | SDK Managers |
| `/sdk/manager/nvm` | `/sdk/managers/nvm` | (detail page) |
| `/sdk?tab=managers` | `/sdk/managers` | SDK Managers |
| `/sdk?tab=services` | `/sdk/services` | SDK Services |
| `/sdk?tab=versions` | `/sdk/versions` | SDK Versions |
| `/sdk?tab=overview` | `/sdk` (stays, query kept) | SDK Management |
| `/sdk/managers`, `/sdk/versions`, `/sdk/services` | themselves | as above |

`/sdk` has no `[role="tab"]` elements left — the tabs are gone. `/sdk/versions` renders.

Overview buttons (checked once the overview had data — see QA-097):
`SDK Managers` → `/sdk/managers`, `Software Installer` → `/sdk/software-installer`,
`Services` → `/sdk/services`. A Quick Navigation manager tile (Conda) → `/sdk/managers/conda`,
and an SDK tile (Python) → `/sdk/python`.

Two things that do not match the claim, neither of them a regression of this issue:

- There is **no Versions button on the overview**. `/sdk/versions` is reachable only from the SDK
  sidebar's Navigation list. Worth a product call if the overview is meant to be the hub.
- `/sdk/installations` is not a route and not a 404 either: it loads the SDK shell with an empty
  main pane and no heading. Pre-existing gap in the catch-all from QA-058; noting rather than
  filing, since nothing links to it.

Blocking finding on this route, filed separately: a cold load of `/sdk` never finishes loading —
**QA-097**.

Shots: `.claude/qa/shots/UX-014-after-sdk-overview-1280.png`,
`.claude/qa/shots/UX-014-after-sdk-overview-loaded.png`, `.claude/qa/shots/sdk-managers-1280.png`.

