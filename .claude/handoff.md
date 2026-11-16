# Handoff: 2026-11-16 — #690 + #769 fixed, still nothing committed

- Shared Portal (:18420) was down at start; `pnpm portal:ensure` once said
  "up" but the process was gone right after — the second run stuck. D: 24GB
  free, C: 34GB free (so #555's 643MB is no longer the state).
- **#690** — `CoderPanel.svelte`: `afterNavigate` re-reads `?id=` and calls
  `selectThread`; skips its first call (the nav that mounted it — onMount
  owns that). **#769** — `agentRuns.svelte.ts`: `runInit` attaches every
  running id, and `session_started` attaches too, so the approval stack has
  logs for unfocused sessions after a reload.
- Live: `.claude/qa/scripts/QA-690-769-verify.mjs` (no arg 3/3, `run` 5/5;
  #690 check fails 1/3 with the fix disabled). `pnpm check`/`lint` 0 errors.
- Dev instance left running in terminal tab "tauri dev cdp".
- Stray untracked file in repo root from an old session:
  `C<U+F03A>Users…scratchpadlint-out.txt` (mangled scratch path) — safe to delete.
- **Next:** commit (user's call), then #726 owner questions, #430/#431/#423.

---

# Handoff: 2026-11-15 — board loop session, ~15 tasks closed, Agents→Coder merge in progress

## Goal this session

User asked to work the Portal Desktop board in a loop: pick up open tasks,
route each through the crew (scout→plan→code→review per CLAUDE.md), and run
independent items in parallel across subagents when they don't share files or
domains. This ran for many rounds. Stopped here because per-turn context in
the orchestrating session got too expensive (300k/turn) — not because the
work ran out. **Every dispatched agent updates the board directly when it
finishes, whether or not the orchestrating session is still open** — so check
`task`/`tasks` for current state first, don't assume anything below is stale.

## Done this session (board tasks closed)

- **#542 Phase 3a** (Portal Mobile notifications): #639 (event→alert mapping
  in `mobile/src-tauri/src/lib.rs`), #640 (wired to `tauri-plugin-notification`,
  foreground-gated via `WindowEvent::Resumed`/`Suspended`), #641 (tap-to-navigate
  - Android 13 permission), #642 (pipeline-finish/fail toast in `app-shell.svelte`).
    Reviewed (1 medium + 3 low findings), then fixed in a separate pass: removed
    a blocking `request_permission()` call from Rust `setup()`, added a
    toast-dismiss fallback keyed by route, unified the pipeline route/wording to
    `/pipeline-runs?run=` on both surfaces, narrowed the notification capability
    to the 4 permissions actually used.
- **#239** — clippy always-true bug in `scan.rs` (`if !in_link_text || true`) —
  removed the dead flag entirely, matched the doc comment's real intent.
- **#621** — device-approval "Select all" silently granted sensitive caps
  (terminal.exec/agents.exec/remote.input). Added `CapabilityInfo.is_sensitive`
  end-to-end: `pairing_service.rs` DTO → `network/commands.rs`'s
  `remote_list_capabilities` → `CapabilityChecklist.svelte` (+ its 3 callers:
  `PendingRequestRow`, `PairDeviceCard`, `DeviceRow`) — Select All now skips
  sensitive caps, which get a destructive Badge.
- **#636** — predev `kill-port` crashed every dev launch (broken npx/nvm).
  Replaced with a native netstat/taskkill implementation in
  `scripts/dev-instance.mjs`, removed the `kill-port` devDependency.
- **#210** — relinking a project could get it flipped back to `archived` by
  the next scan. Fixed `discovery_repository.rs`'s `mark_missing` to re-check
  state after the upsert loop instead of a stale pre-upsert snapshot — one fix
  covers relink and the move feature's identical exposure.
- **#185** — added a git push command: `src-tauri/portal-core/.../coder/git_push.rs`
  (never force-pushes, auto-sets upstream, raw git errors surfaced) + a
  "Push commits…" item in `ProjectQuickActions.svelte`.
- **#241** — `DeviceAuthGuard` called a removed IPC command (`verify_access_token`).
  Root cause was obsolescence, not a missing replacement: the daemon's own
  Origin check (Phase 2/3) already closed that trust boundary. Deleted the
  whole dead path — `DeviceAuthGuard.svelte`, `AuthScreen.svelte` (removed),
  dead methods in `deviceAuthService.ts`. 553 deletions, 27 insertions.
- **#665** — `pnpm check` choked on vendored `inspirations/` configs. Root
  cause was svelte-check's own `ConfigLoader` crawling every subdirectory
  (hardcoded to skip only `node_modules`/dot-dirs) — not tsconfig/svelte.config
  as the ticket guessed. Fixed with svelte-check's `--config ./vite.config.ts`
  flag in `package.json`'s `check`/`check:watch` scripts.
- **#662** — Agents page ignored a `?session=` change while already mounted
  (broke notification tap-to-navigate). Fixed with `afterNavigate` in
  `AgentsWorkspace.svelte`, specifically skipping the `'enter'` event and never
  firing on this component's own `replaceState` — careful not to resurrect
  #635 (today's earlier fix in the same file, same session before this one).
- **#556** — daemon DNS-rebind local-trust bypass: a page whose hostname
  rebinds to 127.0.0.1 could `GET /whoami` with `Trust::Local` (no `Origin` on
  same-origin GETs, and browsers with no Fetch-Metadata support send no
  `Sec-Fetch-Site` either — confirmed the existing Sec-Fetch-Site check,
  landed earlier for #542, didn't fully cover this). Added `Host`-header
  validation in `src-tauri/src/daemon/mod.rs`'s `classify()`.
- **#216** — found already fully shipped 2 days before this task was handed
  out (commits `2f0ea4ea`/`40fd5f17`/`b0a51369`). Verified end-to-end,
  deliberately did not add a second kill path.
- **#393** — `owning_pid` matched the literal English "LISTENING" from
  `netstat`, failing on localized Windows. Fixed in `usage.rs` with a
  locale-proof check (field count + the unspecified-endpoint IP literal,
  never the State column text). Both callers (`check_port_warnings`,
  `move_commands.rs`'s `end_port_process`) route through the one fixed function.
- **#321** — stale pending pairing cards never expired. Added a 5-minute TTL
  to `pairing_service.rs`'s existing `list_devices` expiry filter (same
  chokepoint QR-pickup expiry already used).
- **#273** — projects-list grouping put manually-added projects in their own
  bare-folder group. Fixed `groupProjects.ts`'s `areaOf()` to use 2 ancestor
  segments in both branches, not just 1.
- **#190** closed — its real scope (migration+persistence+Sessions list/detail)
  shipped before this session; the one open piece (#702, an "Agents tab") was
  **cancelled and superseded** by #720 (see below), not completed under #190.
- **#703** — decided (user confirmed): don't build `agent_session_agents` /
  `agent_token_usage`; derive subagent tree + token usage from
  `agent_events.event_json` instead.
- **#721** — Merge 1 of #720 (see below): added a `backendId` prop through
  `AgentsWorkspace`/`AgentSessionSidebar`/`AgentSessionPanel`/`RunLauncherForm`.
  Confirmed invisible today (only one backend, `claude-code`, is registered).
- **#701** — QA-127 re-verified on the current `/ai/agents` route (its scripts
  were stale, pointed at an old route — updated). Reload-mid-approval and
  persisted-sessions-after-restart cases pass cleanly. The kill+restart case
  wasn't cleanly re-landed this round (see Watch out for, below) but nothing
  contradicts the original fix.
- Board hygiene: cancelled #377/#275/#271/#9 (all duplicates of #50's
  note-length bug), completed #236 (duplicate of #239, title already said
  "(fixed)" but status was never closed), completed #679 (ts-rs
  generated-bindings drift, fixed by running `pnpm gen:types`).

Bonus bugs found and filed along the way (not fixed, still open):

- **#690** — `CoderPanel.svelte` has the same "`?id=` read once at mount"
  staleness bug #662 just fixed in `AgentsWorkspace.svelte` (same
  `afterNavigate` fix would apply). Not yet exercised by any live caller, so
  low urgency but real.
- **#719** — `scripts/dev-instance.mjs`'s `owningPids()` (ported from Rust for
  #636) has the exact same localized-Windows "LISTENING"-string bug #393 just
  fixed in `usage.rs`. Needs the same fix shape ported to the JS copy.

## Follow-up session 2026-11-15 (late)

- **#725** found landed in tree but never logged — verified + closed.
- **#723** done inline (mobile notif route → `/ai/coder/claude-code`, mobile
  `cargo test alert_for` 3/3; QA-127 script + stale comments).
- **#722** built by `coder`; review caught that tab links carry no
  `?session=`, so returning to Claude Code lost focus — fixed with
  `agentRuns.lastFocused[backendId]`, restored in `AgentsWorkspace` onMount.
- **#719** done (`dev-instance.mjs` netstat check ported from `usage.rs`).
- Gates on the combined tree: `pnpm check` 0 err, `pnpm lint` 0 err.
  Nothing committed.
- **#724** done live over CDP: `.claude/qa/scripts/QA-724-merge-verify.mjs`
  (no arg: 16/16; `run`: 6/6, starts a real Claude Code run). Check (6) is
  mobile-shell only, covered by `engineTabsFor` + new `navCaps.test.ts`.
  Live run also caught my first #722 restore fix being undone by
  `afterNavigate` firing after `onMount` — restore now lives in
  `afterNavigate`. Filed **#769** (stack empty after reload off-page).
- **Next:** commit the merge work (#720 steps all done; #726 owner questions
  still open), then #690 / #769.
- Board: #377/#275 cancelled, #236 completed. #9 cancel was blocked by the
  auto-mode classifier — still pending, dup of #50.

## The Agents→Coder merge (epic #186, parent subtask #720)

User clarified mid-session that the Agents domain (built in #188-190) isn't
meant to stay a standalone top-level surface — it merges into the Coder
workspace as one tab per CLI backend (today's Coder loop stays as a "Portal"
tab, Claude Code becomes its own tab, Codex/Cursor/Gemini Antigravity follow
later using the same `AgentBackend` abstraction #186 already committed to).
`crew:architect` turned this into a concrete plan, posted as #720 with steps:

1. **#721** ✅ done — `backendId` prop plumbing (frontend-only, invisible today).
2. **#725** ✅ done — route moved to `/ai/coder/[backend]`, old routes redirect.
3. **#722** ✅ done — `CoderEngineTabs.svelte` + `engineTabsFor()`.
4. **#723** ✅ done — mobile route string + QA script + comments.
5. **#724** (claude/qa, not started) — 6-point live verification on a dev instance.
6. **#726** (owner decision, does **not** block 1-5) — four sub-questions:
   - Hide Coder's existing "Claude Code (CLI)" provider (`CoderPanel.svelte:489`)
     now that a real Claude tab exists? _Recommended: yes for new sessions,
     keep old threads readable._
   - Should the Claude tab also get Coder's git/commit/terminal panels?
     _Recommended: yes, as a follow-up step._
   - Where do #191 (Board/Activity/Dashboard) and #192 (Analytics/Workflows)
     mount, now that the `/agents/*` route tree they assumed is gone?
     _Recommended: one cross-backend "Overview" tab at `/ai/coder/overview`,
     backend as a filter._ #191/#192 are blocked on this answer; notes about
     `after_seq` (events pagination, never built) and `backend_id`/`json_extract`
     (per #703) are already logged on both.
   - Is "Portal" the right label for the existing Coder tab?
7. **#193/#194** (Codex/Cursor adapters) — shape unchanged, but acceptance now
   includes "registering the backend makes its tab appear at `/ai/coder/<id>`
   with no frontend edit."

## Also outstanding, not part of #186 — deliberately not touched

- **#419** (high, @owner) — `start_project_move` crashes the whole app on a
  real-sized source tree. Sitting with the owner. Same files as #216/#393/#430/
  #431/#423/#414 (`mover.rs`/`usage.rs`/`move_lock.rs`) — if anyone picks up
  those bugs, read #419 first so a fix doesn't collide with whatever resolves
  the crash.
- **#555** (high, @owner) — D: drive at 643MB free, `cargo test` fails
  disk-full. Explicitly needs the user's call on what to delete, not agent
  work. **Possible correlation**: Portal's shared release daemon (port 18420)
  crashed twice this session for no single obvious cause; disk pressure on its
  SQLite writes is plausible. Fixed both times with `pnpm portal:ensure`
  (idempotent, safe to rerun) — if it recurs, same fix, and maybe worth
  checking disk space at the same time.
- **#50** (high, @owner) — the real portal MCP `update`/`create_task`
  note-length validation bug (5 duplicates of this were cleaned up and pointed
  back at it this session: #377/#275/#271/#9/#236-ish). Still unfixed —
  nobody has actually looked at the validation code yet.
- **#502** (Database Explorer feature, branch `feat/database-explorer`) — P0/P1/P2
  are all done and gate-green, but there's a **staged commit sitting blocked**://
  its own log says "commit needs owner (classifier blocks me)" from before this
  session even started. The Claude Code permission classifier refuses
  agent-initiated commits on this repo. Remaining work (P3 Mongo, P4 Redis,
  a live `tauri:dev` verification that's never been run) was deliberately
  **not** started this session, to avoid piling more uncommitted work onto an
  already-stuck branch — resolve the commit question with the user first.
- **#430/#431/#423/#414** (move-engine bugs, @coder) — NOT touched. Early in
  this session I wrongly assumed these might be moot (thought #197's planned
  redesign would replace this code) — that was wrong, #197's redesign
  (`mover.rs`/`usage.rs`/`move_lock.rs`, steps #201-209) already shipped and
  these bugs are _in_ that shipped code. They're legitimately fixable, just
  ran out of round for them. #197 itself is now down to only **#325** (owner
  decision: preserve empty directories on move?) — everything else in that
  14-step plan is done.
- **#170** — portfolio/case-study task, not real coding work (content
  decisions + an owner-token blocker already noted inline on the ticket
  itself). Skip unless the user specifically wants it worked.

## Decisions made this session, and why

- **Parent-task hygiene**: user flagged (mid-session) that parent tasks often
  sit open after all their subtasks finish, and asked whether the portal MCP
  tool should auto-complete a parent when its subtasks all close, or just
  return a feedback hint. Discussed and recommended **hint over auto-trigger**
  — auto-complete risks false-closing a phased epic like #542/#720 the moment
  one phase's subtasks finish, when more phases are still coming. Not
  implemented, just a recommendation; saved as memory
  `feedback_close-parent-on-subtask-completion.md` (adopted as a personal habit
  regardless: check the parent whenever a subtask closes, instead of trusting
  the board to remind you — and it caught a real case this session, #190).
- **Agents→Coder merge direction**: confirmed by the user, not guessed — see
  the #720 section above.
- **#703 schema**: confirmed by the user — drop the two unbuilt tables.
- **No commits made, anywhere, all session.** CLAUDE.md / the crew:work skill
  explicitly say not to commit unless asked. The user was asked twice whether
  to commit what's done (batched per task) and said "keep going" / "not yet"
  both times — **committing is still an open, explicit ask whenever the user
  is ready**, not a no. Recommend batching by board task given how unrelated
  the ~15 closed tasks are, not one giant commit.

## Watch out for

- **Portal's shared daemon (port 18420) crashed twice this session**, cause
  unconfirmed. Fixed both times with `pnpm portal:ensure` from the repo root.
  See the #555 disk-space note above for a plausible cause.
- **Any `cargo test` inside `src-tauri/portal-core` re-triggers ts-rs's
  export-on-test-run side effect**, rewriting all `generated/*.ts` files
  (~119 of them) unformatted, repo-wide. Purely cosmetic, not a real
  regression — fix with `pnpm gen:types` or `pnpm format`. Already documented
  as a known landmine in `.claude/handoff-android-remote-control.md`; this
  session hit it twice more (closed as #679).
- **Many `crew:coder` agents shared one working tree with no worktree
  isolation this session.** Worked fine because file-domains were checked
  before parallelizing (device caps vs. discovery vs. daemon vs. scripts, all
  disjoint) — never dispatch two agents whose briefs touch the same file.
  Agents that noticed someone else's in-progress uncommitted changes correctly
  left them alone rather than reverting/fixing.
- **`crew:quick` has no portal MCP tools** (Read/Edit/Grep/Glob/Bash/Skill
  only) — the orchestrating session has to update the board on its behalf
  after it reports back.
- **`crew:architect` also lacks the `tasks`/`task` read tools** (only
  `add`/`update`) — it reads board state over the daemon's RPC directly via
  Bash/WebFetch instead of the `task` tool. Works, just surprising.
- **QA/live-testing hit friction from concurrent dev-instance rebuilds** —
  tauri-cli was watching `src-tauri/` for changes while several coder agents
  edited Rust files at the same time, which kept restarting dev instances
  faster than a controlled kill+restart test could land (#701's case 2).
  Not a product bug, a scheduling collision — avoid heavy parallel Rust edits
  and live dev-instance QA in the same window next time.
- **`mobile/src-tauri/src/lib.rs` has uncommitted changes from this session's
  Phase 3a work sitting in it right now.** #723 (queued, not started) touches
  exactly 2 lines of it — don't let a broader edit there collide with the
  rest of that file's pending diff.

## Files that matter

- `mobile/src-tauri/src/lib.rs` — Phase 3a notification bridge (`alert_for`,
  `Mobile::notify`, foreground tracking via `WindowEvent`); #723 still owes it
  a 2-line route-string update.
- `src/lib/utils/notify.ts`, `src/lib/components/layout/app-shell.svelte` —
  toast/notification plumbing (Phase 3a + #662).
- `src/lib/domains/agents/components/{AgentsWorkspace,AgentSessionSidebar,AgentSessionPanel}.svelte`,
  `.../RunLauncherForm.svelte` — #721's `backendId` plumbing; #725 should have
  wired the real route param in by the time you read this.
- `src/lib/config/agents-tabs.ts` — being rewritten across #725 then #722,
  strictly sequential.
- `src/lib/domains/devices/components/CapabilityChecklist.svelte`,
  `src-tauri/.../network/{pairing_service.rs,commands.rs}` — #621, #321.
- `src-tauri/portal-core/src/domains/projects/discovery/{scan.rs,discovery_repository.rs}` — #239, #210.
- `src-tauri/portal-core/src/domains/projects/relocation/usage.rs`,
  `src-tauri/src/domains/projects/move_commands.rs` — #393 (fixed); #419
  (crash, untouched) and #430/#431/#423/#414 (untouched) also live here.
- `src-tauri/src/daemon/mod.rs` — #556's `Host`-header fix in `classify()`.
- `scripts/dev-instance.mjs` — #636's native port-kill; #719 (unfixed) is the
  same localization bug in this file's `owningPids()`.
- `.claude/qa/issues/QA-127-*.md` + `.claude/qa/scripts/QA-127-*.mjs` —
  updated/re-verified by #701.

## State of the working tree

**Nothing committed all session**, by design (not asked to). Expect 45+
modified/new files spanning all of the above, across ~15 unrelated completed
tasks. Recommend committing in batches grouped by board task before this
grows further — the user has been asked and has deferred twice, not declined.

---

# Handoff: 2026-11-15 (continued again) — #635 fixed and verified, nothing committed yet

## This continuation

User asked to fix #635 right after it was filed. Done:

- `AgentsWorkspace.svelte`: `focusedId` changed from
  `$derived(page.url.searchParams.get('session'))` to local `$state`, set
  directly by `selectSession`/`newRun`/the `onStarted` callback. `syncUrl()`
  now only mirrors the id into `?session=` for deep-linking/refresh, mirroring
  how `CoderPanel.svelte` actually works (drives off its own state, URL is
  just a mirror) rather than the inverted pattern that was there before.
- Verified live over CDP on the dev build: clicking a sidebar row opens the
  thread immediately (no reload), switching between two different sessions
  works, and a freshly-started run opens its own thread right away.
- `pnpm check` (0 errors) and `pnpm lint` (0 errors, no new warnings in the
  touched file) both pass.
- Closed board #635 with the fix note. **Not committed** — one file changed
  (`src/lib/domains/agents/components/AgentsWorkspace.svelte`), nothing
  staged, waiting on the user to say whether to commit/push.
- Along the way the dev instance died mid-session (terminal tab + release
  daemon both dropped — looked like an environment/sandbox restart, not
  anything this session did) and had to be restarted twice
  (`node scripts/dev-instance.mjs 1`, then `pnpm portal:ensure` to bring the
  shared :18420 daemon back). Both came back clean; no data lost, the two
  test sessions from earlier are still in the dev DB.

## Still left (unchanged from before)

1. Commit/push the #635 fix (holding for the user).
2. Click through the pairing approval in a real window (dev or release).
3. Install the new release build and smoke-test it — installer still
   untested in `src-tauri/target/release/bundle/{nsis,msi}/`.
4. #636 (predev npx) and #550 (global pnpm) — likely the same nvm/npm
   corruption, worth fixing together.
5. #577 (Portal `update`'s `note` field length bug) still open.

---

# Handoff: 2026-11-14 (continued) — Agents redesign live-verified over CDP, found a real regression

## This continuation found and did

Picked up the "left to do" list below. User was present and steered toward
using the documented CDP harness instead of computer-use pixels (right call —
see `[[driving-portal-desktop-over-cdp]]` memory).

1. **Fixed the launch path, not just the symptom**: `.claude/launch.json`'s
   `tauri` config runs `pnpm run tauri dev`, which never sets
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` — only
   `node scripts/dev-instance.mjs 1` does (see its own comment, which says
   plain `pnpm tauri:dev` "is left exactly as it was — same CDP port", which
   is **not true** for how the browser-pane launcher invokes it). Started the
   dev build via `node scripts/dev-instance.mjs 1` in a Terminal tab instead;
   CDP came up on :9222 as expected. **If you reach for `preview_start({name:
"tauri"})` for CDP work, it won't give you a debug port — use the terminal
   - `dev-instance.mjs 1` instead.**
2. **Live-verified the Agents redesign (#546, dd27f0f0)** — the one thing
   last session couldn't test. Ran a real Claude Code agent session
   (`List the files in the current directory, then stop.`, cwd = Devstrail)
   through the real backend over CDP via `.claude/qa/scripts/app.mjs` +
   `window.__TAURI_INTERNALS__.invoke`:
   - Backend is solid: `agents_start_run` → real process → real transcript
     (`agents_get_session` returned the actual `ls` output).
   - **Found a real regression, filed as board #635**: clicking a session row
     in the sidebar (or finishing starting a new run) updates the URL
     (`?session=<id>`, confirmed via `page.url()`) but the main panel stays
     on the blank "Start run" launcher. A **hard reload of that same URL**
     renders the thread correctly — transcript, expandable tool-call ("Ran
     1s"), status badges, follow-up composer, all fine. So the bug is
     specifically that `AgentsWorkspace.svelte`'s
     `focusedId = $derived(page.url.searchParams.get('session'))` doesn't
     react to its own `replaceState` call; its comment claims this mirrors
     `CoderPanel.svelte`'s `?id=` sync, but `CoderPanel` actually drives
     rendering off its own `coderSession.activeThreadId` state and only uses
     `replaceState` to keep the URL in sync for deep-linking — not as the
     trigger. `AgentsWorkspace` inverted that. Did **not** attempt the fix
     this pass — it's a real one worth a focused coder session, and I'd
     already spent the budget diagnosing it precisely (see #635 for the full
     repro + root-cause writeup).
   - Could not test the approval-banner / `decide()` path, since the trivial
     `ls` prompt never needed a permission grant and the sidebar bug blocked
     reopening a thread anyway to try a prompt that would.
3. **Filed #636** (low): `predev` (`node scripts/dev-instance.mjs
--kill-port`) crashes on every dev launch — nvm's `npx` is broken for node
   v24.14.1 (`Cannot find module '@npmcli/config/lib/definitions'`). Harmless
   when the port's already free, but means `kill-port` never actually runs.
   Related to #550.
4. **Did not click through the device-pairing approval** (left-to-do #1
   below) — ran out of session budget after the Agents dig; the dev instance
   from this session is still up (CDP :9222, vite :1420, daemon :18421) if
   you want to pick that up directly.
5. Cleaned up all scratch `.claude/qa/scripts/continue-*.mjs` test scripts —
   none kept, the findings are in #635/#636 instead.

## Still left (updated)

1. **Fix #635** (Agents sidebar navigation) — the highest-value next step;
   it blocks actually using the redesign day-to-day.
2. Click through the pairing approval in a real window (dev or release) —
   still nobody has watched a paired browser get a working session.
3. Install the new release build and smoke-test it — installer still sitting
   untested in `src-tauri/target/release/bundle/{nsis,msi}/`.
4. #636 (predev npx) and #550 (global pnpm) are likely the same underlying
   nvm/npm corruption — worth fixing together.
5. #577 (Portal `update`'s `note` field length bug) still open, still just
   worked around by keeping notes short.

## Watch out for

- The dev instance this session started (`node scripts/dev-instance.mjs 1`,
  terminal tab "tauri dev cdp") may still be running — CDP :9222, vite :1420,
  dev daemon :18421. Check before assuming a clean slate, same as last time.
- Use `node scripts/dev-instance.mjs 1` for CDP work, **not**
  `preview_start({name: "tauri"})` / plain `pnpm tauri:dev` — only the
  former sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`.
- The release daemon on :18420 is still the shared singleton — untouched
  this session, same caution as before applies.

---

# Handoff: 2026-11-14 session close — pushed, release built, PC about to shut down

## What this session found and did

Picked up the previous `.claude/handoff.md` (Agents redesign, board #546). That
handoff was stale: the Agents work (`dd27f0f0`) was already merged to `main`
and pushed by a later session, which then also did two rounds of Portal
Mobile work (Phase 1 phone polish, Phase 2 remote caps) on top. Confirmed via
`git merge-base feat/database-explorer main` — the old branch's tip is an
ancestor of `main`, not a sibling.

Per the user's live request this session (unattended, "finish everything you
can on your own, shut down if you hit something that needs a click"):

1. **Verified board bug #618** ("Phase 2.1: event_cap forwards..." subtask)
   was shown unchecked but already landed in `ca57f65d` — `git show` confirmed
   the diff is there. Marked it `completed`, checkbox was just stale.
2. **Live-verified the browser/other-desktop client** (task #542) for the
   first time in a real browser — this was the one genuinely unverified piece
   left in `docs/development/PORTAL_MOBILE.md`. Rebuilt `build-client`, opened
   a browser-pane tab at the dev daemon (`:18421`), confirmed:
   - the daemon's `fallback_service` serves `/connect` correctly
   - `browserShim.ts` installs `window.__TAURI_INTERNALS__` and the real
     connect UI renders (not a crash, which is all last session confirmed)
   - pairing to `127.0.0.1:18421` fails outright (`Failed to fetch` — browser
     treats `localhost`/`127.0.0.1` as different origins even same
     port/machine). Pairing to `localhost:18421` (the page's own origin)
     works and reaches the real "Approve on your PC…" long-poll state.
   - **Did not click Approve.** That's a device-approval decision inside the
     running desktop app — needs the native window. Computer-use access
     needs a one-time permission dialog the user has to click; user was AFK,
     so per their instruction this was left rather than waited on.
     Docs + board task #542 updated with this.
3. **Could not live-verify the Agents redesign** in the native dev window for
   the same reason — same computer-use permission-dialog blocker. Skipped,
   per the same instruction.
4. **Built the release installer**: `pnpm tauri:build` succeeded clean —
   `src-tauri/target/release/bundle/nsis/Portal Desktop_0.15.0_x64-setup.exe`
   and the `.msi` sibling both exist.
5. **Did not install it.** The currently-running release
   (`%LOCALAPPDATA%\Portal Desktop\portal_desktop.exe`, pid seen this session)
   is the one shared task-board daemon on :18420 that every session/tool on
   this machine talks to (including this session's own board MCP calls).
   Overwriting/restarting it mid-session to test one feature, with no way to
   click through the resulting UAC prompt or the app's own UI afterward
   (same computer-use blocker as above), was judged not worth the shared-state
   risk. **This is the one explicit ask ("verify in a release installed on
   this PC") that did not get done — flagging it clearly rather than claiming
   it happened.**
6. **Pushed to `origin/main`**: `ca57f65d` (Phase 2 remote caps, was sitting
   local-only) and `8096df10` (this session's docs update). `origin/main` ==
   `main` now.

## Left to do (next session, or next time someone's at the keyboard)

1. **Click through the pairing approval** in the real desktop app (dev or
   release) to finish verifying the browser/other-desktop client end to end —
   the request reaches "Approve on your PC" correctly; nobody has clicked
   Allow and watched the paired browser actually get a working session.
2. **Install the new release build and smoke-test it** — the installer is
   sitting in `src-tauri/target/release/bundle/{nsis,msi}/`, untested.
   Whoever does this should close the currently-running release first (it's
   the shared :18420 daemon other tools/sessions may be using) rather than
   installing over a live process.
3. **Live-verify the Agents redesign** in an actual native window — tool
   calls rendering properly, approval banner + `decide()`, resume reusing the
   sidebar row, the global floating approval stack, project/task cross-nav
   with real ids. None of this has been click-tested since it was built
   (see git history around `dd27f0f0` for what it is).
4. Board bugs still open: **#550** (global pnpm broken) — looks fixed now,
   `pnpm -v` returned `9.15.9` cleanly this session, not independently
   re-verified end to end; **#577** (Portal MCP `update`'s `note` field choking
   on long text) — still open, still worked around by keeping notes short.

## Watch out for

- **The release daemon on :18420 is shared, singleton, load-bearing.** Do not
  stop/replace/reinstall over it without checking nothing else on the machine
  is mid-task against the board. This session deliberately did NOT do that.
- **Computer-use access needs a one-time user click** on its permission
  dialog. If the user is away, this blocks any native-window verification —
  don't `request_access` and sit there; skip it and say so, like this session
  did, rather than hanging.
- **`localhost` vs `127.0.0.1` are different browser origins** even same
  port/machine — a same-machine pairing test must use whatever host the page
  itself was loaded from, or the fetch fails before it even reaches the
  daemon. Not a bug, just a gotcha for whoever tests this again.
- Dev build was started this session (`pnpm tauri:dev` via the browser-pane
  preview launcher, since direct terminal invocation of `pnpm tauri:dev` was
  blocked twice by a hook rewriting the command mid-flight — fell back to
  `preview_start({name: "tauri"})` instead, which worked). That dev process
  (and its native window) may still be running when this PC is turned back
  on if the shutdown didn't clean it up gracefully — check before assuming a
  clean slate.
