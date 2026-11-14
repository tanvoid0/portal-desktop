# Handoff: Agents redesign (board #546) — built, committed, not pushed

## Goal

User rejected a first pass at "redesign the Agents page" that was just a moved route + a
grouped table. What they actually wanted: the Agents feature (external CLI agent backends —
Claude Code today, more later) to feel like a real Claude Code IDE / this app's own Code tab —
project-grouped session sidebar, a live-streaming transcript, new-run and viewing-a-session
being the same view, and cross-navigation to/from Projects and Tasks. Board task #546
(`mcp__plugin_crew_portal__task 546` for full history — already marked `completed` there).
Branch `feat/database-explorer` (yes, that's the branch name even though this work is unrelated
to the database explorer — it was just the active branch when this session started).

## Status: done, gated clean, committed. Not pushed. Not live-verified against a real backend.

Commits on `feat/database-explorer` (newest first, not pushed):
- `b9cb8bf2` — tiny: adds the `.gitignore` entries the previous commit's message promised but
  forgot to actually stage.
- `2576a838` — untracks 45 QA screenshots + `.qa424-src/` that `git add -A` swept into the main
  commit by mistake (see "Watch out for" below — this bit me once already, don't repeat it).
- `dd27f0f0` — **the actual feature**, 115 files, +2516/-950. Also carries forward unrelated
  in-progress task-board/mobile-docs edits that were already sitting in the working tree when
  this session started (not mine, just along for the ride since the user asked to commit
  everything in the tree).

### What's implemented

Built in 5 steps via `crew:architect` (plan) then `crew:coder` subagents (build), each step
gated before the next started. Full plan/rationale is in the board task's note history if you
need the "why" behind a specific choice — the summary:

- **Backend** (`src-tauri/portal-core/src/domains/agents/`): new migration
  `m20261114_000049_add_agent_session_links.rs` adds nullable `title`/`task_id`/
  `backend_session_id` to `agent_sessions`. Event ring buffer now carries `seq` (`AgentEventSeq`
  ts-rs type), so `agents_get_events` and the live `agents://event` payload are both
  `{run_id, seq, event}`. New `resume_session` (reuses the SAME session row, not a new one —
  confirmed user decision) and `delete_session`, both refuse while the session is live. New
  commands `agents_resume_session`/`agents_delete_session` in
  `src-tauri/src/domains/agents/commands.rs`, both capped `agents.exec`. `agents_start_run` now
  takes `task_id` and runs with **`isolate_settings: false`** (confirmed decision — a run loads
  the user's real `~/.claude/settings.json`/project `CLAUDE.md`, same as Coder already does;
  this flipped from `true` in the original code).
- **Tool-call rendering** (`src/lib/domains/agents/utils/eventsToMessages.ts`): native Claude
  Code tool names (`Bash`, `Read`, `Write`, `Edit`, `Grep`/`Glob`, `LS`) now map onto Coder's
  synthetic shape (`run_command`, `read_file`, etc.) so `ToolCallCard.svelte` renders them
  properly instead of a bare name + raw JSON dump — **this was the biggest reason the old UI
  didn't look like Claude Code**. Also: `tool_result.is_error` prefixes `"Error: "`,
  `thinking_delta` accumulates into a thought block, subagent messages (non-null
  `parent_tool_use_id`) stay out of the main feed, and `eventRowsToEvents()` lets a historical
  session replay through the same reducer as a live one.
- **State** (`src/lib/domains/agents/state/agentRuns.svelte.ts`, new): module-singleton
  multi-run store, one shared `agentsApi.onEvent` listener for the app's life (replaces the old
  per-page single-run `agentRun.svelte.ts`, which is deleted). `attach(id)`, `viewFor(id)`,
  `runningIds`, `pendingCountFor(id)`, `pendingLaunch` (handoff slot for "open Agents with this
  cwd/prompt/taskId pre-filled"), and thin `start/resume/sendUser/decide/interrupt/cancel`
  wrappers.
- **UI** (`src/lib/domains/agents/components/`, new): `AgentsWorkspace.svelte` (the shell,
  `?session=` URL-driven), `AgentSessionSidebar.svelte` (project-grouped via
  `agents/utils/groupSessions.ts`, live status overlay), `AgentSessionPanel.svelte` (launcher
  when nothing's focused, live transcript + composer when something is — reuses Coder's
  `CoderFeed`/`ApprovalBanner` as shared leaves, does NOT reuse `CoderPanel` itself, which is
  welded to Coder-only state), `AgentSessionTimeline.svelte`, and `AgentApprovalStack.svelte` (a
  **global floating stack** — confirmed user decision — mounted in `app-shell.svelte`, shows
  pending approvals for sessions you're not currently looking at, from any page in the app).
  `/ai/agents` is the only real route now; old `/ai/agents/run`, `/ai/agents/sessions`,
  `/ai/agents/sessions/[id]` are `+page.ts` redirect stubs. Agents is the 3rd tab on `/ai`
  (Home/Code/Agents) — the standalone top-level "Agents" sidebar nav entry is gone.
- **Cross-navigation**: a Project's card/row/quick-actions menu and its detail page
  (`src/routes/projects/[id]/+page.svelte`) can launch an Agents run scoped to that project's
  path, and show its last 5 sessions. A Task's menu/manager can do the same, pre-filling a
  prompt from the task (`src/lib/domains/tasks/components/TaskManager.svelte`'s
  `handleLaunchAgents`), and `TaskInfoCard.svelte` lists sessions tagged with that `task_id`.
  `TaskDetail.svelte`'s existing "resume session" action now checks for a matching
  `backend_session_id` in Agents first, falling back to its old `claude --resume` terminal path
  only if none is found. The focused session's header in `AgentSessionPanel.svelte` shows a
  project chip (linking to `/projects/<id>`) and a task chip (linking to `/tasks/<id>`) when
  resolvable.

### Verification done

- **Rust**: `cargo check`, `cargo clippy --all-targets` clean (zero new warnings in the agents
  domain). `cargo test agents` from `src-tauri/portal-core/` — 15/15 passing, including 3 new
  resume-guard unit tests (already-live-but-row-says-exited, row-says-running,
  no-backend-session-id). `cargo test --lib rpc` from `src-tauri/` — 15/15, confirming the two
  new commands' capability wiring didn't break the exact-cap-list test.
- **Frontend**: `svelte-check` — 0 errors across 3949 files (134 pre-existing warnings elsewhere,
  unrelated). `eslint` clean. `vitest` — all agents-domain test files passing (groupSessions,
  eventsToMessages, agentRuns — ~20 tests across those three).
- **Visual, by me, in a plain browser (no real Tauri backend)**: confirmed `/ai/agents` renders
  the sidebar (search box, "New run", project groups) and the launcher centered in the main
  panel, no crash, matches the target shape. **This is the only visual check that happened.**
- **NOT done**: no click-through against a real running app with real data. Nobody has seen a
  live session actually stream, a tool call actually render with the new mapping, resume
  actually work, the global approval stack actually float, or the project/task cross-nav links
  actually navigate correctly with real ids. All of that needs the real Tauri backend (SQLite +
  IPC), which a plain browser preview can't provide (`invoke` isn't injected outside the actual
  webview).

## Left to do

1. **Live-verify in the real app.** This is the big one — see "Watch out for" below, the native
   dev binary was locked by a stale process at the end of this session and I could not rebuild
   it. Start fresh (`pnpm tauri:dev` if `pnpm` works for you, otherwise see the workaround
   below), start a real Agents run, and check: tool calls render properly (not raw JSON), the
   approval banner appears and `decide()` actually unblocks the run, ending and resuming a
   session reuses the same sidebar row, the global approval stack shows up when you navigate
   away from a blocked session, and the Project/Task cross-nav links actually work with real
   ids (not just "doesn't crash with empty data", which is all I could confirm).
2. **Push, or don't** — nothing's pushed, no PR opened. User hasn't asked for either yet.
3. Two bugs filed on the board, both assigned `owner`, both still open:
   - **#550** — global `pnpm`/`corepack` is broken on this machine (`MODULE_NOT_FOUND` on its
     own entrypoint). Every subagent and I worked around it with
     `./node_modules/.bin/<tool>.CMD` directly. Not fixed, not investigated beyond confirming
     it's broken — outside this task's scope, but it'll bite the next session too.
   - **#577** — the Portal MCP `update` tool's `note` field throws `"Task title must be less
     than 200 characters"` for any note over ~190 chars (validating note length against the
     title limit, looks like a copy-paste bug in the tool itself). Worked around by splitting
     notes into short ones. Not fixed.
4. **Minor, not done, probably fine to skip**: no per-project-group "New run" button in the
   sidebar (only one global one at the top) — the original architect plan mentioned per-group
   buttons, the implementing agent judged the global one sufficient and I didn't push back.

## Decisions made and why

- **Agent Platform (the separate `agent-platformd` product at
  `D:\projects\production\ai\agentic-ai\agent-platform`) does NOT get a new `AgentBackend`
  adapter.** Originally planned, then dropped after research showed Portal's own Coder tab
  already drives the exact same agent-platform coder-chat loop, with *stronger* permissions
  (gates file writes, not just shell commands). Building a second, weaker front-end to the same
  thing was pointless. The user's own framing: agent-platform is meant to stay a pure
  AI-provider layer (OpenAI-compatible API multiplexer, eventually a standalone production
  server for other apps), not a Portal-specific agent runner. If anyone proposes this again,
  point them at this paragraph and at Coder's `platform_stream.rs`/`coder_loop.rs`
  equivalent in the agent-platform repo.
- **Resume reuses the same session row**, not a new one each time (user's explicit call,
  over AgentMonitor's "new row per resume" alternative). This is why `resume_session` has to
  guard against double-liveness (row says `"exited"` but it's still in the live `runs` map) —
  see the 3 new Rust unit tests.
- **Coder and Agents share leaf components (`CoderFeed`, `ApprovalBanner`,
  `ResponsivePanel`/`PageContainer`) but NOT the shell** (`CoderPanel`, `CoderSessionList`) —
  those are welded to Coder-only state (`coderUi`, `coderWorkspaceStore`, git/terminal panels).
  User said Agents may fold into Coder as another "engine" eventually; this leaf-sharing
  structure is specifically meant to make that merge cheap later without forcing it now.
  Nothing else needs to change for that to stay true — don't "simplify" by inlining Coder's
  shell into Agents, it'd cut off that path.
- **Global floating approval stack** (`AgentApprovalStack.svelte`, mounted in `app-shell.svelte`)
  was a deliberate addition beyond the sidebar badge, explicitly requested — a blocked run
  should be visible from any page, not just when Agents is open.
- **`isolate_settings` flipped from `true` to `false`** for Agents runs — explicit user call,
  "feels like the real Claude Code IDE" was read to mean the user's real `~/.claude/settings.json`
  and project `CLAUDE.md` should apply, same as Coder already does, not a sandboxed subset.

## Watch out for

- **`git add -A` is dangerous in this repo right now.** This session did it once (user's
  explicit instruction: "commit the working tree changes") and it swept in 45 new QA screenshots
  plus a scratch fixture dir (`.qa424-src/`) that the repo's convention says to leave untracked.
  Caught it by actually reading a stale `.claude/handoff.md` from an *earlier* session that had
  documented this exact convention — don't skip reading this file's "watch out" sections in
  whatever state you find them. First fix attempt (`git rm -r --cached .claude/qa/shots
  .qa424-src`) was **way too broad** — it untracked 1368 pre-existing, legitimately-tracked
  screenshots going back through the whole project's QA history, not just the new ones. Caught
  via `git commit` output listing far more `delete mode` lines than expected — **always eyeball
  a commit's file count/list against what you think you're committing before trusting it, even
  for a "simple" fix.** Had to `git reset --soft HEAD~1` + `git reset` to undo, then redo with
  the exact ~48-file list instead of a directory-wide `rm --cached`. Both paths are now in
  `.gitignore` (`.claude/qa/shots/`, `.qa424-src/`), so this specific mistake can't recur via
  `git add -A` again — but the lesson generalizes: a broad git operation on a shared dir needs
  its file count checked before you trust it, not after.
- **A stale native dev-app window (`portal_desktop.exe`) was left running at the end of this
  session**, pointed at a vite server that had since been stopped. It held a file lock that made
  `cargo test agents` (the bin target) fail with `Access is denied` removing the old exe — worked
  around by testing the `portal-core` lib directly (`cd src-tauri/portal-core && cargo test
  agents`) instead of going through the bin. Killing the stale process via `taskkill` was denied
  by the Bash tool's permission classifier ("Destructive or Unwanted Actions") every time it was
  tried. If you hit the same lock, either ask the user to close the window themselves, or just
  test against `portal-core` directly like this session ended up doing — don't fight the
  classifier on `taskkill`.
- **Global `pnpm`/`corepack` is broken in this dev environment** (see bug #550 above) —
  `MODULE_NOT_FOUND` on pnpm's own entrypoint, `npx` too. Every check in this session ran via
  `./node_modules/.bin/<tool>.CMD` directly (`vitest.CMD`, `eslint.CMD`, `svelte-check.CMD`,
  `prettier.CMD`, `vite.CMD`, `tauri.CMD`) instead of the `pnpm` scripts. A temporary
  `.claude/launch.json` entry (`web-local`, pointing `runtimeExecutable` at the local
  `vite.CMD` binary with an absolute path) was added and removed twice this session for browser
  previews — if you need it again, same trick, just remember to remove it when done (it's not
  meant to be a permanent fixture, `pnpm`'s own `web`/`tauri` configs are still the "real" ones
  once pnpm itself is fixed).
- **This working tree had unrelated in-progress work already in it when this session started**
  (task-board components, `docs/development/PORTAL_MOBILE.md`, etc. — not mine, from whatever
  was running on this branch before). It's now folded into `dd27f0f0` because the user explicitly
  said to commit "the working tree changes" as one commit. If that turns out to be wrong (the
  user actually wanted it kept separate), it'll need a `git revert`/manual split — ask before
  assuming either way.
