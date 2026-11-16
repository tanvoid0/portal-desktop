# Handoff: 2026-11-15 — Action runs viewer / live output / Stop / MCP-aware stop (board #691)

## Goal

User: "Why can't I see the details of action runs? Why can't I see the logs live and
interact (stop, pause, anything)?" Then mid-way: "we also need a context menu when
right-clicking the runs list, and if stopped/paused the MCP server needs to know if an MCP
requested the run", then "it's not using the width of the screen properly".
User chose fixes 1–4 (viewer, Stop, live output, Run in terminal) and did **not** pick pause.

## State: code done + verified, NOTHING COMMITTED (two repos)

The Portal working tree also holds **other sessions' uncommitted work** (mobile/, devices,
app-shell.svelte, AgentsWorkspace, coder, etc.). Commit only the files below, on a new branch
(user is on `main`; global rule: branch first, no Claude attribution trailer).

### portal_desktop files (this work only)

- `src-tauri/portal-core/src/domains/scripts/services/script_execution_service.rs` — runner:
  raw-chunk UTF-8-safe output (`decode_utf8_chunk`, `pump`), stdout+stderr in one
  arrival-ordered buffer that is also what's persisted (the `[stderr] ` prefix is gone),
  `kill_tree(pid)` (async taskkill /T /F; unix `process_group(0)` + `kill -KILL -- -pgid`),
  `cancel_requested` set, Stop = flag + kill; the output task saves output **then** writes
  `cancelled` / "Stopped by user" (clean exit 0 ⇒ stays success). Child registered before
  `update_pid` and the flag re-checked (spawn race). Leftover rows from an earlier session
  are closed as "Not tracked by this Portal session — marked stopped" (no kill of a
  maybe-reused pid). Startup sync also closes `pending` rows. After a Stop, pumps get 2 s
  then are dropped so an orphan holding the pipe can't keep the run "running". 3 unit tests.
- `src-tauri/src/domains/scripts/commands.rs` — `ExecuteScriptParams.triggered_by` (default "user").
- `src/lib/domains/actions/runOutput.ts` (+ `.test.ts`) — `toTerminalText` (strip ANSI, `\r`
  redraw keeps last state), `runRequester` ("mcp:x" → "MCP · x").
- `src/lib/domains/actions/runActions.ts` — stop/rerun/delete/copy shared by detail + menu;
  Stop on an MCP-requested run confirms first.
- `src/lib/domains/actions/components/ActionRunDetail.svelte` — loads by id, polls status +
  live buffer 1 s while active, Stop / Re-run / Run in terminal (`EmbeddedTerminal`, records
  via `recordInteractive`) / copy; `terminal` is bindable.
- `src/lib/domains/actions/components/ActionRunList.svelte` — rows + right-click menu via
  `$lib/components/menu/ContextMenuList.svelte`; `onChanged(deletedId?)`.
- `src/lib/domains/actions/components/ProjectActionsPanel.svelte` — uses the two components;
  old inline poll/detail removed; Stop is not a failure (no toast, no Diagnose card);
  cards grid when stacked, detail beside history ≥2200px.
- `src/lib/domains/actions/{types.ts,failure.ts,actionsService.ts}` — step status
  `'cancelled'`; `toFailure` returns null for it; a Stop ends a multi-step run.
- `src/routes/pipeline-runs/+page.svelte` — now lists **script_executions** (it read
  `pipeline_executions`, which nothing writes any more); project by exact path; polls only
  `get_running_script_executions`; selection via `goto(url)` (QA-158: `pushState` doesn't
  update `page.url`); terminal tied to run id (`terminalFor`).
- `src/routes/pipeline-runs/[id]/+page.svelte` — run on its own page (stable link for a
  project or session): back to all runs, project button (exact-path match), Re-run navigates
  to the new run's page. `ActionRunDetail` has `showPageLink` (open-page icon link), the list
  menu has "Open page", and crew `run`/`run_status` output names `/pipeline-runs/<id>`.
  Verified live: rail link -> page renders. Breadcrumb shows the raw uuid (cosmetic).
- `src/lib/domains/mobile/navCaps.ts` — `/pipeline-runs` → cap `actions`.
- `src/app.css` — `--content-max-width-full: max(112rem, 85vw)` (was fixed 112rem; on the
  user's 3440px monitor content went 1792 → 2924px). Affects every page.
- `.claude/qa/issues/QA-158-*.md` — marked fixed with verification.

### crew plugin (`D:\projects\production\ai\plugins\crew`, separate repo)

- `mcp/portal.js` — new tools `run` (execute_script with `triggeredBy: mcp:<clientInfo.name>`,
  waits ≤ wait_seconds, reports **STOPPED BY THE USER** distinctly) and `run_status`;
  `terminalText` + selftest assert. `README.md` table rows. `.claude-plugin/plugin.json` 0.2.3 → 0.3.0.
  `node mcp/portal.js --selftest` passes.

## Verified

svelte-check 0 errors · eslint 0 errors (prefer-const warnings are repo-wide style) ·
vitest `src/lib/domains/actions` 52/52 · `cargo check` ok · runner tests 3/3.
Live QA on dev instance 2 (`node scripts/dev-instance.mjs 2`, vite :1421, CDP :9223 — still
running): run/live output/`\r`/ANSI, Stop (5→0 node procs, cancelled + output kept, no
failure card), Re-run, Run in terminal, menus, global page, filters, 700px + 3440px no
overflow, QA-158 fix (click opens detail). Reviewer agent findings all addressed except
below.

## Left to do

1. Commit on a branch (two repos), no attribution trailer. User was asked; no answer yet.
2. Install crew 0.3.0: `claude plugin update crew@<marketplace>` + restart.
3. Release build so daemon :18420 records `triggered_by` and has the new Stop (old daemon
   ignores `triggeredBy` and reports a stop as `failed`).
4. Untested: Delete confirm / deleting the selected run clears detail, Copy command/output,
   Run in terminal from the Action Runs page, long project names.
5. Known leftovers (not done, by choice): `app-shell.svelte:~264` still routes the dead
   `pipeline-execution-update` toast to `/pipeline-runs?run=<pipeline id>` (file had other
   uncommitted edits); Windows orphans that outlive their parent survive Stop (Job object
   per run is the upgrade); pause not built.
6. QA saw once (not reproducible): reload → click running row → Stop → run kept going, detail
   showed an older run. Watch for a selection/reload race in `ProjectActionsPanel`.

## Things that bit

- Portal's SvelteKit `goto` has no `noScroll`/`keepFocus` (newer API: `replace`, `shallow`, …).
- `portal-core` isn't a workspace member: run its tests from `src-tauri/portal-core`.
- `.claude/handoff.md` belongs to another session (agents #635) — this file is separate.
- Bash heredocs with `'''` break in this tool; put patch scripts in a file.
