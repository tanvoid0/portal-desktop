# Coder revamp — plan and status

Goal: one Coder workspace where the provider (Portal engine, Claude Code, Codex, Cursor, any ACP
agent) is a property of a thread, not a tab you navigate to. The hub (Rust) owns every thread, so
the desktop window, Portal Mobile and a browser all see the same thread live and can catch up
after a gap. Every provider gets the same feed, a diff view, and controls that match what it can
really do.

Borrowed from T3 Code (`pingdotgg/t3code`, MIT): its UX and its server-authoritative,
event-sourced thread model. Not borrowed: its transport (Effect RPC); the daemon's `/rpc` +
`/events` already do that job (see [REMOTE_ARCHITECTURE.md](REMOTE_ARCHITECTURE.md)).

PCORE = `src-tauri/portal-core/src/domains`. Coder FE = `src/lib/domains/coder`.

## Status

Board: epic #1907.

| phase | board | what                                                     | size | state             |
| ----- | ----- | -------------------------------------------------------- | ---- | ----------------- |
| 0     | —     | land #1836 (sidebar) and #1845 first                     | —    | in flight (#1836) |
| 1     | #1916 | server-owned threads: one replayable event log, commands | L    | planned           |
| 2     | #1908 | one thread list, provider + model picked in composer     | M    | planned           |
| 3     | #1909 | honest per-provider capabilities, mid-thread switching   | M    | planned           |
| 4     | #1910 | diffs for every provider via git checkpoints             | M-L  | planned           |
| 5     | #1911 | shared composer and feed polish                          | L    | planned           |
| 6     | #1912 | ACP adapter, provider handoff, multi-model fan-out       | L    | planned           |
| 7     | #1913 | Portal engine as an `AgentBackend`                       | XL   | decide after 1-6  |

Phase 1 comes first because phases 2-6 all read thread state, and that state has to live on the
hub before a second client can use it. Phases 3 and 4 are independent (adapters vs git module)
and can run in parallel after 2.

## Where we are (2026-11-27)

Two stacks share one frame (`CoderShell.svelte`):

- **Portal engine**: `/ai/coder` → `CoderPanel` → `state/coderSession.svelte.ts` (1489 lines) →
  `PCORE/coder/service.rs` (2792 lines) → agent-platform SSE `POST /api/v1/coder/chat/stream`.
  Tools run on the host (`coder/tools.rs`). Has agent modes, permission modes and rules,
  mid-session model switch, queue, retry/edit, sub-agents, multitask, per-hunk accept/reject.
- **CLI agents**: `/ai/coder/[backend]` → `AgentsWorkspace` → `PCORE/agents/service.rs` →
  `AgentBackend` adapters `claude_code.rs`, `codex.rs`, `cursor.rs` → normalized `AgentEvent`
  (`agents/events.rs`) → `agents/utils/eventsToMessages.ts` → the same `CoderFeed`.

Portal is already a hub: all commands are reachable over `POST /rpc/*` and every event over the
`GET /events` WebSocket (`src-tauri/src/daemon/`), with pairing and per-device caps. The Coder
does not use that well yet:

- Portal-engine events carry no sequence number (`coder/service.rs` has none) and only go out
  live. A client that opens a thread mid-turn misses everything before it. A lagging client gets
  `events://lagged` and resumes at the present (`daemon/mod.rs:702`), with no way to catch up.
- Thread runtime state lives in the client: the follow-up queue (`messageQueue`), the partial
  streaming text and the running flag are fields in `coderSession.svelte.ts`. Reload the webview
  or open the thread on a phone and they are gone or different.
- Two clients can both act: approve twice, send twice. Nothing deduplicates commands.
- The CLI-agents stack is halfway there: per-run `seq`, a ring buffer, the `agent_events` table and
  `agents_get_events` catch-up (`agents/service.rs:99-132`).

UX pain points:

- Provider is a route tab (`src/lib/config/agents-tabs.ts` `engineTabsFor`), so threads are split
  by engine and you pick the engine before you pick the work.
- `agentCapabilities(_backend)` (`config/capabilities.ts:36`) returns one stripped set for all
  three CLIs: no permission modes, no changes, no retry, no queue, model fixed at launch. The
  adapters already take a permission mode at start (`codex.rs:129` `policy_for_mode`), so the UI
  under-reports what they can do.
- CLI threads get no agent diff: `AgentsWorkspace.svelte:296-298` passes `changes={[]}`.
- Claude `list_models` returns empty (`claude_code.rs:62-68`), so its model is free text.
- QA-085: switching engine does not retarget an existing thread.

## What to take from T3 Code

| T3 Code                                                                   | take? | where               |
| ------------------------------------------------------------------------- | ----- | ------------------- |
| Server-authoritative threads: clients render projections, never own state | yes   | Phase 1             |
| Event log + projections committed in one transaction; replay after a gap  | yes   | Phase 1             |
| Pure decider (`command → events`) with side effects run after commit      | yes   | Phase 1             |
| Command ack = intent committed; one result even when two clients act      | yes   | Phase 1             |
| Queue and approvals held on the server; approve from any device           | yes   | Phase 1             |
| Provider + model picker in the composer, grouped by provider              | yes   | Phase 2             |
| "Working" section in the thread list (running, waiting on approval)       | yes   | Phase 2             |
| Typed capabilities per provider; UI never branches on provider name       | yes   | Phase 3             |
| Never emulate a missing permission mode; plan mode only if it emits plans | yes   | Phase 3             |
| Turn checkpoints as hidden git refs → diff and revert for any provider    | yes   | Phase 4             |
| Diff sources: this turn / uncommitted / vs base branch                    | yes   | Phase 4             |
| Queue vs Steer follow-ups while a turn runs                               | yes   | Phase 5             |
| Tool-call verbosity: All / Hide successful / Zen                          | yes   | Phase 5             |
| Select diff lines → review comment → composer chip                        | yes   | Phase 5             |
| `@` files, `/` commands, attachments, Ctrl+K thread palette, shortcuts    | yes   | Phase 5             |
| Generic ACP adapter (one adapter, many agents)                            | yes   | Phase 6             |
| Switch provider mid-thread via budgeted history handoff                   | yes   | Phase 6             |
| One prompt → several models, one worktree each                            | yes   | Phase 6             |
| Effect RPC transport, 35-package split                                    | no    | `/rpc` + `/events`  |
| Bounded replay window                                                     | later | if a log grows big  |
| Snooze / settle lifecycle, PR watcher that wakes the agent                | later | own tasks if wanted |
| Multi-account provider instances (`CLAUDE_CONFIG_DIR` per instance)       | later | only if needed      |

## Phase 0 — clear the deck

Land #1836 (sidebar star/reorder/delete, uncommitted in the tree now) and #1845. Phase 2
rewrites the same files: `ProjectWorkspaceList`, `utils/sessionList.ts`, `state/coderUi.svelte.ts`.

## Phase 1 — server-owned threads, one replayable event log

Detailed design and slices (#1925-#1935): [CODER_THREAD_LOG_DESIGN.md](CODER_THREAD_LOG_DESIGN.md).
Where it differs from the bullets below (normalised vocabulary instead of "`AgentEvent` plus
variants", a startup importer instead of a one-off migration, no receipts table), the design wins.

The foundation. Every client (desktop webview, Portal Mobile's shell, a browser over the daemon)
renders the same thread from the hub, joins mid-turn, and catches up after a gap.

1. **One append-only thread log for both stacks**: `(thread_id, seq, kind, payload, command_id,
at)`, `seq` per thread. Vocabulary = `AgentEvent` plus the Portal-only variants (file
   change / hunk, sub-agent, multitask, mode and permission changes, queue changes, approval
   decided-by). Token deltas stay live-only, not persisted; the projection holds the current
   partial message so a late joiner sees it.
2. **Projections in the same DB transaction** as the append: thread summary (provider, model,
   project, status running / waiting / idle / error, pending approval, last activity, title) and
   the message list. Projections are the source of truth. Existing `coder_thread` rows and agent
   sessions are migrated once into projections (history read-only, no synthetic replay).
3. **Snapshot + catch-up**: `coder_thread_snapshot(id)` returns the projection at `seq` S. The
   client subscribes to `coder://thread-event {thread_id, seq, event}`; on a gap, a reconnect or
   `events://lagged` it calls `coder_thread_events_since(id, seq)`. One TS reducer applies events;
   the desktop and mobile shells share it because they run the same frontend.
4. **Commands carry a client `command_id`**; the hub dedups it (receipt stored with the event).
   Approvals resolve first-wins; the second client gets "already resolved", and the event says
   which device decided.
5. **Runtime moves to the hub**: follow-up queue, pending approvals, running state. The
   `coderSession.svelte.ts` store shrinks to view state plus the reducer.
6. **Shape**: pure `decide(&ThreadState, Command) -> Result<Vec<ThreadEvent>>` and
   `apply(&mut ThreadState, &ThreadEvent)`. Side effects (spawn or message the provider, run a
   host tool, git checkpoint, notification) run after commit and feed results back as commands.
   One writer per thread (a per-thread task owning its state) so two clients never interleave.
7. **Remote**: nothing new in transport. Everything goes over `/rpc` + `/events` with pairing and
   caps; `coder_approve` and the other write commands are already in the sensitive set
   (`src-tauri/src/rpc.rs:341`). A thread entering "waiting on approval" raises a notification
   (the existing inbox) so a phone can act on it.

Done when: reload the webview mid-turn → identical thread, queue intact; a second client (browser
over the daemon, or Portal Mobile) opens the same thread mid-turn and catches up, including after
a forced `events://lagged`; approve from both → applied once, the loser told; Rust decider/apply
tests and vitest reducer replay tests (events in → state out) for both stacks.

## Phase 2 — one thread list, provider in the composer

- `/ai/coder` shows one sidebar served from the Phase 1 thread-summary projection (one RPC over
  both stacks, so a phone gets the same list), grouped by project (#1836's grouping), provider
  badge per row, **Working** section on top (running, or waiting on an approval / ask-user).
- The new-thread composer gets a provider + model picker grouped by provider: Portal catalog +
  `agents_list_backends` + each backend's `list_models`. The provider is fixed at first send
  (switching comes in Phase 6).
- Selecting a row mounts the matching body inside the one `CoderShell`. Engine tabs go; a provider
  filter chip replaces them. Overview stays as a page, linked from the sidebar.
- `/ai/coder/[backend]` and existing deep links (`sessionPath`) redirect into the merged page.

Done when: one list on desktop and mobile shell; a Claude, a Codex and a Portal thread started
from the same composer; old deep links still open; `pnpm check && pnpm lint`.

## Phase 3 — honest per-provider capabilities

- Add `AgentBackend::capabilities()` returning a ts-rs `ProviderCapabilities`; it replaces the TS
  stub in `config/capabilities.ts`. The Portal engine reports its own set. Capabilities travel in
  the thread snapshot so every client renders the same controls.
- Fill in what each CLI really supports:
  - **Claude Code**: permission modes default / acceptEdits / plan / bypassPermissions; switch
    mid-session over stream-json `control_request` (we already send `interrupt` that way,
    `claude_code.rs:277`; `set_permission_mode` / `set_model` need checking against the installed
    CLI). Model list = aliases (default, opus, sonnet, haiku) plus custom ids.
  - **Codex**: approval + sandbox already mapped (`policy_for_mode`); models from RPC; reasoning
    effort; per-turn overrides on `turn/start` (check the params).
  - **Cursor**: whatever `cursor-agent` flags allow; otherwise stays launch-only.
- The UI reads capabilities only: permission chip, model picker (catalog vs launch-only), effort
  picker, plan mode only where the provider emits plans. A stored mode the provider does not
  offer falls back to its default; nothing is emulated.

Done when: Claude permission mode and model switch mid-thread in the real app; a capabilities
unit test per adapter.

## Phase 4 — diffs for every provider (git checkpoints)

- Rust, next to `coder/git_status.rs` / `diff.rs`: at turn start and at turn end, write a
  checkpoint without touching the branch or the user's index — temp `GIT_INDEX_FILE`,
  `git add -A`, `write-tree`, `commit-tree`, `update-ref refs/portal/checkpoints/<thread>/<turn>`.
  It is a Phase 1 side effect; the ref lands in the log as an event, so every client sees it.
- Diff view gets a source picker: **This turn** / **Uncommitted** / **vs base branch**. Reuses
  `ChangeHunk` rendering.
- **Revert turn** restores files from the turn-start ref. Files only; the conversation is rewound
  only where a capability says the provider can, and the UI says "files only" otherwise.
- Portal's per-hunk accept/reject stays (a Portal capability).
- Refs are deleted with the thread.

Done when: a Claude or Codex turn shows its own diff and revert restores the tree; Rust tests on
a temp repo (checkpoint, diff, revert, untracked files, HEAD and index unchanged).

## Phase 5 — shared composer and feed polish

- Follow-ups while running: **Queue** (held on the hub, sent at turn end) or **Steer** (sent now,
  where the provider accepts mid-turn input). Ctrl+Enter does the other one.
- Feed verbosity setting: All / Hide successful / Zen. Approvals, questions and errors always
  show. A filter in `utils/feedBlocks.ts`.
- `@` file mentions (from `list_dir`), `/` commands (provider commands + ours), image and file
  attachments where the capability allows.
- Select diff lines → review comment → composer chip → sent as one message (needs Phase 4).
- Ctrl+K palette over threads (title, project, branch, message text). Check whether an existing
  palette (`k8s-navigation/components/CommandPalette.svelte`) can be lifted first.
- Shortcuts: model picker, permission picker, new thread, jump to Working.
- Thinking blocks and usage / limit display the same across providers.

## Phase 6 — more providers, cross-provider moves

- `agents/acp.rs`: a generic Agent Client Protocol adapter (JSON-RPC over stdio), strictly to
  spec, no per-agent hacks. Brings Gemini CLI, OpenCode, Goose and others, registered by command
  in settings. Tests from replayed real runs, like the existing adapters (`codex.rs:695`).
- Provider settings: detected / version / signed-in per backend (passive; `detect()` exists),
  custom model ids.
- Switch provider or model mid-thread: start the new backend session seeded with a budgeted
  transcript handoff (no reasoning or raw tool state), recorded as an event in the same thread.
  Fixes QA-085.
- Fan-out: pick several models → one thread + worktree each, same prompt, diffs side by side.
  Builds on multitask and `coder/worktree.rs`.

## Phase 7 — decide later: Portal engine as an `AgentBackend`

After Phase 1 both stacks already write one log and share one reducer, so this is narrower than
before: whether the Portal engine becomes one more `AgentBackend` (`agents/portal.rs` wrapping the
agent-platform loop) so there is a single provider seam on the Rust side too. Host tool
execution, sub-agents, multitask and hunks would move behind it. Only worth it if keeping two
producers of the same events is what slows changes down.

## Open decisions (owner)

1. Engine tabs: remove (badge + filter chip instead) or keep as filters? Plan assumes remove.
2. Phase 7: defer (plan assumes) or commit now?
3. First ACP agents to target: whichever are installed and signed in here.

## Risks and unknowns

- Phase 1 migration: existing `coder_thread` message blobs and `agent_sessions` must map into the
  new projections without losing history. Plan: one-way migration, old tables kept until it has
  run on real data.
- Large tool output in the log: store big payloads once (truncated in the event, full body
  fetched on demand) so snapshots stay small for a phone.
- How a phone is woken for an approval: check what Portal Mobile supports today
  ([PORTAL_MOBILE.md](PORTAL_MOBILE.md)) before Phase 1 step 7.
- Claude CLI `set_model` / `set_permission_mode` control requests: verify on the installed CLI
  before building Phase 3 UI on them.
- Codex `turn/start` per-turn overrides: verify.
- Checkpoint cost on large repos (`git add -A` into a temp index walks the tree): measure on the
  biggest project; skip large untracked files past a size cap if needed.
- Steer on Claude: a user line sent mid-turn may just queue inside the CLI; check.
- Neither app was looked at visually for this plan. It comes from our code and T3 Code's docs and
  source, not from screenshots.
