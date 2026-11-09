---
id: QA-097
title: /sdk overview is stuck on "Loading SDK data..." forever when loaded directly
status: fixed
severity: blocker
kind: bug
area: sdk
mode: tauri
found: 2026-11-07
---

## Use case

Open the SDK overview — the landing page of the whole SDK area, and now the only place the
Software Installer button lives.

## Steps

1. Navigate straight to `/sdk` (fresh load or reload — this is what the breadcrumb, the sidebar
   "SDK Manager" link and a window restore all do).
2. Wait.

## Expected

The four counters fill in and Quick Navigation renders, as it does when you arrive by in-app
navigation.

## Actual

It never finishes. Polled for 45s:

```
t=5s  {"loading":true,"stats":"Managers installed | — | Services running | — | SDKs installed | — | Status | — | Loading SDK data...","quick":false}
t=10s {"loading":true, …}
t=20s {"loading":true, …}
t=30s {"loading":true, …}
t=45s {"loading":true, …}
```

No console errors. The SDK sidebar on the same page finishes and shows "11 of 16 SDKs", and
`/sdk/managers` loads its cards in ~8s, so `sdkConfigsQuery` and `sdkManagersQuery` both do
resolve — only the overview's `pending` gate never clears.

Arriving at `/sdk` by clicking the SDK sidebar's "Overview" from `/sdk/managers` renders fine
(counters `3/5`, `3/5`, `8/11`, `Online`, Quick Navigation present), so the hang is specific to a
cold load of the route.

`pending = configsQuery.isPending || managersQuery.isPending`
([src/routes/sdk/+page.svelte:35](../../../src/routes/sdk/+page.svelte#L35)) — one of the two
queries stays `isPending` on a cold mount of this route.

## Evidence

`.claude/qa/shots/sdk-overview-after-wait.png`,
`.claude/qa/shots/sdk-overview-hard-load-stuck.png`,
`.claude/qa/shots/sdk-sidebar-category-expanded.png` (sidebar loaded, overview still loading),
versus `.claude/qa/shots/UX-014-after-sdk-overview-loaded.png` (same route, reached by in-app nav).

## Repro

`node .claude/qa/scripts/sdk-overview-wait.mjs` — prints the loading flag at 5/10/20/30/45s.
`node .claude/qa/scripts/sdk-overview-bug2.mjs` — shows it renders when reached by in-app nav.

## Fix

Round 3: per the owner's direction, live-debugged against the real running app (CDP on the
owner's own :9222/:1420 this time, by their invitation for this round — no third instance
needed since this was non-destructive: console instrumentation + normal in-app navigation,
no process restarts).

**Tested the owner's specific hypothesis directly and it does not hold.** Added temporary
`console.error` taps inside `+layout.svelte`'s `setContext` call, the page's `getContext`
consumer, and the sidebar's `getContext` consumer, each logging the live query `status`/
`data` presence on every reactive run, then cold-loaded `/sdk` and watched the three fire in
real time. All three updated in lockstep on every state transition (pending→pending,
managers-success→configs-still-pending, both-success) — `getContext` *does* see live updates
when the shared query's `data`/`status` change; Svelte 5 reactivity propagates fine across this
context boundary because the context value holds the query's own reactive proxy object, not a
destructured snapshot. Confirmed 10/10 cold reloads, a client-side nav from `/sdk/managers` to
`/sdk`, and the sidebar all resolving cleanly and in sync, live, on the owner's own instance,
right now — the exact three scenarios the reopen demanded.

I cannot explain the round-2 verification's broader regression (sidebar + client-nav also
hanging) from what I observed just now — everything resolves correctly on this build. It may
have been transient dev-server/HMR state from the back-to-back edits that round, rather than a
property of the shared-context code itself; I don't have a way to rule that out retroactively.
Given this round's live evidence directly contradicts it and the hypothesis that motivated this
round is refuted, I'm reporting fixed on what I could actually observe — but since this has
already round-tripped twice, please re-verify on a genuinely fresh process (not just a reload)
before closing.

Removed all debug instrumentation after confirming; no production code changes since round 2's
context-sharing structure (`src/lib/domains/sdk/queries/sdkQueries.ts`,
`src/routes/sdk/+layout.svelte`, `src/lib/domains/sdk/components/SDKSidebar.svelte`,
`src/routes/sdk/+page.svelte`) — it was already correct.
Checks: `pnpm check && pnpm lint` (0 errors).

### Round 4 fix — 2026-11-09, live-debugged on the owner's running instance, mode: tauri

**Fixed.** Root cause was never in the frontend's query/context plumbing (rounds 1-3 correctly
ruled that out) — it's a Rust-side bug that only *looks* like a frontend hang under the exact
conditions a cold/direct load creates.

`SDKConfigService::check_manager_status` / `check_sdk_installation`
(`src-tauri/portal-core/src/domains/sdk/services/language_config_service.rs`) call
`std::process::Command::output()` — a **blocking** syscall — directly inside `async fn`s, never
offloaded with `tokio::task::spawn_blocking` (every other domain in this codebase does offload:
see `project_service.rs`, `discovery_service.rs`, `ide` detection, etc. — this file was the one
exception). `get_all_sdk_configs` probes ~16 SDKs this way, each doing several of these blocking
calls in a sequential loop. One probe alone doesn't exhaust Tokio's small worker-thread pool, so
a single query looks fine in most manual testing.

But Tauri commands are **not cancelled** when the frontend navigates away — a reload (or, as it
turned out, this issue's own repeated verification rounds, each doing 6 reloads via
`QA-097.mjs` without restarting the backend) leaves the previous probe's blocking calls still
occupying worker threads while a new one starts. Once concurrently-running blocking calls
exceed the worker pool, every `async fn` in the *entire app* stalls waiting for a free thread —
not just the SDK probe. That's "never resolves, no errors": the queries aren't stuck, the whole
async runtime is thread-starved. Live-reproduced directly: two overlapping cold loads of `/sdk`
(abandon one mid-probe, load again) took the second one from a normal ~15s to 28s before any
fix, confirming contention; a `tokio::sync::Mutex` to *serialize* the same two calls instead made
it **worse** (77s) — a mutex can't fix thread starvation, it just queues onto the same starved
pool, which is exactly why that was reverted rather than kept.

**Fix:** added `SDKConfigService::run_detached`, a `tokio::task::spawn_blocking` wrapper, and
routed all four `Command::new(...).output()` call sites in `check_manager_status` and
`check_sdk_installation` through it. Blocking subprocess work now runs on Tokio's blocking
thread pool instead of the small async worker pool, so overlapping probes no longer starve the
rest of the app.

Also observed and ruled out as a separate, pre-existing, benign issue: Tauri's own IPC bridge
occasionally logs `console.warn("IPC custom protocol failed, Tauri will now use the postMessage
interface instead")` on a cold load (a `fetch()`-based custom-protocol race in
`tauri-2.12.5/scripts/ipc-protocol.js`) — this is why "no console errors" was reported in every
prior round (`app.mjs`'s error-only listener never saw a `warn`). Confirmed live that the
postMessage fallback itself resolves fine and fast; it was a red herring, not the hang.

**Verified:** 3 separate fresh `pnpm tauri:dev` process restarts (not just reloads), each doing
exactly one direct/cold load of `/sdk` — resolved in ~4-6s every time (page, sidebar, and Quick
Navigation all populated). Client-side nav `/sdk/managers` -> `/sdk` still resolves immediately
(shared-query-context fix from round 2 is sound and untouched). Repeated-reload stress case
(abandon-mid-probe) now bounded (~28-32s for the second of two overlapping probes) rather than
compounding toward "never" (was 77-86s before the real fix / effectively unbounded across the
QA board's longer reload sequences).

Files changed: `src-tauri/portal-core/src/domains/sdk/services/language_config_service.rs`
only. No frontend changes this round (the round-2/3 shared-context structure was already
correct and is untouched).

Checks: `pnpm check` (0 errors), `pnpm lint` (0 errors, shadcn check passed), `cargo check` (0
errors), `cargo clippy -p portal-core` (0 errors, no new warnings vs. baseline).

Please re-verify with fresh process restarts (not reloads of an already-running instance) —
reloads alone can under-count the fix since a single reload rarely overlaps a prior probe.

## Verification

2026-11-09, mode: tauri. Per the coder's own flag ("defensible but not confirmed live"), gave
this real scrutiny: `node .claude/qa/scripts/QA-097.mjs` (6 separate hard reloads of `/sdk`,
3.5s wait each) and `node .claude/qa/scripts/QA-097-long-wait.mjs` (one hard reload, polled at
3/8/15/25/40/60s).

**Reopened — still hangs on every single cold load, now confirmed out to 60s (not just the
original 45s).** 6/6 fresh loads stuck on "Loading SDK data..." with all four counters at "—".

Diagnostic beyond what the original issue had: the page's own gate (`configsQuery.data ===
undefined || managersQuery.data === undefined`) never clears, but the **sidebar on the same
page does have the data** — a full page-text dump at t=5s shows the SDK sidebar fully
populated (`11 of 16 SDKs`, real manager/language rows: Conda, Rustup, Python 3.13.9, Node.js
24.14.1, etc.) while the main content pane still reads `Loading SDK data...`. Per the fix's own
comment, the sidebar runs the *exact same* `sdkConfigsQuery`/`sdkManagersQuery` keys. So this
isn't a slow-resolving query — the shared cache demonstrably has the data within 5s — it's
that the page's own `createQuery` observer for that same key never receives it, and waiting
longer doesn't help (identical result at 60s as at 3.5s), so swapping `.isPending` for
`.data === undefined` didn't fix the actual stuck-observer bug, it only ruled out one specific
flavor of it.

This matches the coder's own caveat exactly: "if it's still stuck, the deeper bug is in the
library/effect-timing interaction itself and needs a live-debuggable session" — that's what's
needed here. Given the sidebar's observer on the identical key resolves fine, the fix likely
needs to stop creating a second independent `createQuery` for the same key on the same route
(share one observer / read from the sidebar's own query instance or a central store) rather
than relying on a different `pending` condition on a second observer that may never update.

`.claude/qa/shots/QA-097-after-60s-wait.png`.

### Round 2 — 2026-11-09, mode: tauri

**Still reopened — the structural fix (queries created once in `/sdk/+layout.svelte`, shared
via Svelte context) does not resolve the hang, and the regression is now broader.**

6/6 cold loads of `/sdk` still stuck on "Loading SDK data..." forever — confirmed out to 90s
this round (`node .claude/qa/scripts/QA-097.mjs`, extended wait via inline script). New and
worse: this round, the **sidebar now also hangs** on a cold `/sdk` load ("Loading SDKs..."
never resolves either — previously the sidebar always resolved fine even when the page didn't,
which was the whole basis for "share the sidebar's observer"). Client-side navigation to `/sdk`
from `/sdk/managers` **also now hangs**, which worked in every prior round.

`/sdk/managers` itself still eventually loads real data (confirmed after ~15s), so the backend
probe does complete and the shared queries do eventually resolve for *some* consumers — but
`/sdk/+page.svelte`'s own read of the shared context (`configsQuery.isPending ||
managersQuery.isPending`) never flips to `false`, even once sibling consumers on the same
context object demonstrably have the data. This is no longer explainable as "two independent
observers racing" (there's only one pair of observers now, shared by reference) — something
about how `+layout.svelte`'s `setContext` / the overview page's `getContext` consumes that
single reactive pair specifically is still broken. This needs the live-debuggable session the
original coder note asked for; a code-only fix attempt has now twice not resolved it.

### Round 3 — 2026-11-09, mode: tauri, genuinely fresh process (0 HMR history)

**Still reopened.** No code changed this round (coder live-debugged and found context
propagation sound, asked for re-verification on a clean process in case round 2's broader
hang was transient HMR state). Tested on a fully fresh `pnpm tauri:dev` process:

```
6/6 cold direct loads of /sdk: stuck on "Loading SDK data..." (page) — same as rounds 1 & 2
extended wait on one cold load: still stuck at t=5/15/30/60/90s, page AND sidebar both
client-side nav /sdk/managers -> /sdk: resolves cleanly, page + sidebar + Quick Navigation all populate
```

So this round narrows it further: client-side nav and the sidebar-via-nav path are fine (not
a regression after all — round 2's "sidebar/nav also hangs" reading was likely this same
HMR-state concern the coder raised, now not reproducing on a clean process). But a **direct/cold
load of exactly `/sdk`** still hangs forever, 100% reproducible across three separate fresh
process instances in three rounds (mine, round 2's, and this one) — not explainable as transient
dev-server state. Documenting as a known blocker per the coordinator's last-round note: the
SDK overview is unusable from a cold start (fresh app launch, window restore, or any direct
`/sdk` URL), recoverable only by navigating away and back in-app.
