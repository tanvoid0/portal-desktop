---
id: QA-127
title: "Agents > Run: live run state is purely in-memory on both ends — reloading/revisiting the page abandons a pending approval forever, and the matching session sits \"running\" in Sessions permanently with no way to resolve it"
status: open
severity: major
kind: bug
area: agents/run, agents/sessions
mode: tauri
found: 2026-11-11
---

## Use case

A user starts a run from Agents > Run, it pauses on "Approval required", and
the user navigates away (or the page reloads — WebView2 can do this on its
own, same scenario QA-119 already documented for the coder domain). They
expect to either get the banner back so they can decide, or for the run to
be cleanly marked as no longer active. Separately, anyone later browsing
Agents > Sessions expects "running" to mean something is actually still
running.

## Steps

**A — reload loses a pending approval:**
1. Isolated instance (`CDP_PORT=9223`). Agents > Run, any cwd, prompt: "Use
   the Write tool to create a file named qa126-write-test.txt... Then stop."
2. Wait for the "Approval required" banner (confirms QA-126's fix: shows
   `Write` + the full path).
3. Reload/revisit the `/agents/run` route (e.g. `page.goto` to the same URL,
   or Playwright's `open()` helper navigating to the route again — same
   effect as an F5 or a WebView2-initiated reload).
4. Observe the Run tab, then check Agents > Sessions for that run's row.

**B — pre-existing stuck rows (found, not caused, by this sweep):**
1. Agents > Sessions, no action needed — two rows were already present:
   session `cc09a67f-2805-40ca-afbf-4fee3e727501` (`claude-code`,
   `D:\projects\production\portal\portal_desktop`, started "Today, 16:56")
   and one more started "Today, 17:21" — both status `running`, `Ended: —`,
   from *before* the currently-running app instance's own process start
   (confirmed via `Get-CimInstance Win32_Process`: `portal_desktop.exe`
   PID 55112 created 18:04:24 — both sessions' `started_at` predates that).
2. Open `cc09a67f-...`'s detail page.

## Expected

A: either the banner is reconstructed from backend state after a reload (the
fix QA-119 shipped for the coder domain) so the user can still decide, or the
UI clearly shows the run is gone/unreachable rather than a blank, fresh
"Start run" form implying nothing was ever running.
B: a session whose owning process is long gone should resolve to a terminal
status (`exited`/`interrupted`) — not float at `running` forever with no
indication anything is wrong and no way to fix it from the UI (Sessions is
explicitly read-only, per its own file comment).

## Actual

**A:** After the reload, `/agents/run` shows a completely fresh, empty
"Start run" form — `run.runId` is pure client-side `$state` from
`createAgentRunState()` (`src/routes/agents/run/+page.svelte`), never
rehydrated from the backend on mount. There is no indicator anywhere on this
page that a run is still active. Checking Sessions confirms the backend run
is in fact still alive and blocked: the session sits at `running` with no
`ended_at`. The approval that was pending is now undecidable forever — the
`claude` CLI subprocess sits blocked on stdin indefinitely (verified: its
`cmd.exe`/`claude.exe` process pair was still alive in `Get-CimInstance`
output well after the reload, only going away once I killed it directly via
`Stop-Process` to unblock the semaphore slot it was holding).
Screenshots: `.claude/qa/shots/QA-126-verify-live-banner.png` (banner up,
before) → `.claude/qa/shots/qa126-recheck-state.png` (same route, after
reload: blank form) → `.claude/qa/shots/qa126-sessions-after-navaway.png`
(Sessions still shows that run `running`).

This is the Agents-domain counterpart to QA-119 (already fixed and verified
for `ai/coder`'s claude-cli engine) — `ApprovalBanner.svelte` is literally
shared between the two domains, but `agents/run`'s own page has no equivalent
to coder's `claude_cli_pending` map / `selectThread()` rehydration fallback.
Nothing in `createAgentRunState()` queries the backend for "is there a run
already going that I should reattach to" on mount.

**B:** `cc09a67f-2805-40ca-afbf-4fee3e727501`'s own detail page (not just the
list) also shows `Status: running`, `Ended: Still running` — permanently,
confirmed by reloading the Sessions list well after the owning process is
gone. `AgentRunService::new()` (`service.rs:142-151`) builds a fresh, empty
`runs: Mutex<HashMap::new()>` on every app start with no pass over the DB to
reconcile any `status = 'running'` row left over from a previous process
lifetime — the pump task's own close-the-gap fix (`service.rs:226-234`,
"the session would otherwise sit at status running forever") only covers a
crash *within* the same process lifetime (channel closes, pump task catches
it and persists `interrupted`); it does nothing for the case where the whole
process already died and restarted, since there is no pump task left to
catch anything. Screenshot:
`.claude/qa/shots/QA-127-stale-running-session-detail.png`.

Both A and B are one underlying gap: a run's "is this actually still alive"
state lives only in memory on both ends (frontend `$state`, backend
`HashMap`), and nothing bridges a persisted `running` DB row back to "is
there still something reachable for it" — on a frontend reload (A) or a
backend restart (B).

## Why it matters

A pending approval that's silently abandoned is worse than one that's
rejected: the user never finds out their decision was needed, the run just
disappears from view while actually still blocked and holding a concurrency
slot (`MAX_CONCURRENT_RUNS` semaphore permit) until the app itself is killed.
And a Sessions list that can say "running" about something with zero chance
of ever finishing again misleads anyone using it to check on agent activity.

## Evidence

- `.claude/qa/shots/QA-126-verify-live-banner.png`,
  `.claude/qa/shots/qa126-recheck-state.png`,
  `.claude/qa/shots/qa126-sessions-after-navaway.png` — scenario A sequence.
- `.claude/qa/shots/QA-127-session-after-kill.png` — same session correctly
  flips to `exited` once its process is killed directly (confirms the
  in-process crash path *does* work; it's only the "frontend forgot about a
  still-alive backend run" and "backend process itself is gone" paths that
  don't reconcile).
- `.claude/qa/shots/QA-127-stale-running-session-detail.png` — scenario B,
  pre-existing stuck row's own detail page.

## Repro

Scenario A: `CDP_PORT=9223 node .claude/qa/scripts/QA-126-verify.mjs` (starts
the run, confirms the live banner), then separately navigate `/agents/run`
again on the same page/connection and confirm the form is blank while
Sessions still shows that run `running`.
Scenario B: `CDP_PORT=9223 node .claude/qa/scripts/QA-127-stale-session-check.mjs`
— opens the pre-existing stuck row's detail page and confirms `running`.

## Fix

Root cause (one gap, two symptoms): a run's "is this actually still alive"
state lived only in memory on both ends — the frontend's `createAgentRunState()`
had no persisted run id to rehydrate from on mount, and the backend's
`AgentRunService::new()` built a fresh, empty `runs` map on every process
start with nothing to reconcile it against `agent_sessions` rows left
`status = "running"` from a previous process lifetime.

**A (reload abandons a pending approval):** unlike coder's claude-cli engine
(QA-119), `/agents/run`'s own normalized event ring (`AgentRunService.runs[id].ring`)
already carries the unresolved `permission_request` event verbatim — nothing
strips it, so `eventsToMessages()` derives the pending banner from a backfilled
ring the same way it does for a live run. The only missing piece was a way for
the page to discover, on mount, which run id (if any) is still alive on the
backend to backfill from. Added `AgentRunService::list_running()` (run ids with
a live backend entry right now — cheap/non-sensitive, so `cap = "agents"` not
`agents.history`), exposed as `agents_list_running`, and a `rehydrate()` on
`createAgentRunState()` that calls it on mount: if a live run id comes back, it
reattaches (`ensureListening()` + `getEvents()` backfill) instead of showing a
blank "Start run" form. No new "pending" command needed — the ring backfill
already reconstructs it.

**B (stale `running` rows after a restart):** added
`AgentRunService::reconcile_stale_sessions()` — on startup, after
`AgentRunService::new()`, flips every `agent_sessions` row still
`status = "running"` to `"interrupted"` with `ended_at = now()` (same terminal
status the existing pump-loop channel-closed fix already uses for an
in-process crash). Wired into `src-tauri/src/lib.rs` right after
`agents_service.init_events(...)`, mirroring the existing
`execution_service.reconcile_interrupted()` pattern for pipelines.

Files changed:
- `src-tauri/portal-core/src/domains/agents/service.rs` — `list_running()`,
  `reconcile_stale_sessions()`.
- `src-tauri/src/domains/agents/commands.rs` — `agents_list_running` command.
- `src-tauri/src/lib.rs` — register `agents_list_running`; call
  `reconcile_stale_sessions()` at startup (logs count via `log_info!`/`log_warn!`).
- `src/lib/domains/agents/api/agentsApi.ts` — `listRunning()`.
- `src/lib/domains/agents/state/agentRun.svelte.ts` — `rehydrate()`.
- `src/routes/agents/run/+page.svelte` — calls `run.rehydrate()` in `onMount`.

Checks run: `cargo check` and `cargo clippy -p portal-core --no-deps` from
`src-tauri/` — clean (only pre-existing warnings elsewhere, none in touched
files). `pnpm check` — 0 errors. `pnpm lint` — eslint/prettier/shadcn-ui clean
on every touched file (the one `pnpm lint` failure, `scripts/ensure-portal.mjs`
prettier formatting, is a pre-existing untracked file this change never
touches).

Live verification (2026-11-11, mode: tauri, isolated instance CDP :9223,
rebuilt from the working tree including this fix — tauri-cli's watcher
auto-rebuilt on save, confirmed via matching file mtimes / `cargo run`
process start time):
- Scenario A: `.claude/qa/scripts/QA-127-verify-reload.mjs` — started a run,
  waited for "Approval required" (Write, full path), reloaded `/agents/run`.
  Banner reappeared with the same tool + path (`QA-127-after-reload.png`),
  "Accept once" worked and the run proceeded — the pending decision is no
  longer unreachable after a reload.
- Scenario B: the two pre-existing stuck rows from the issue report
  (`cc09a67f-...`, started "Today, 16:56", and the "Today, 17:21" row) both
  now show `interrupted` with `Ended: Today, 18:30` — the moment the rebuilt
  instance started up and ran `reconcile_stale_sessions()`. Confirmed via
  `.claude/qa/scripts/QA-127-stale-session-check.mjs`'s own target row no
  longer matching its old `running`/`16:56` filter, and a direct read of the
  Sessions list.

No other open QA id shares this root cause (QA-119 is the already-fixed coder
counterpart this mirrors, not an id this also closes).

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
