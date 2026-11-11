# Handoff — `agents` domain review-fix + live-QA loop, all fixes verified, nothing committed

Written 2026-11-11, for someone with **no memory** of this session. Supersedes the previous
version of this file (that was about the Android remote-control merge; see git history —
`ade510ed` — if needed. That work is already committed to `main`, this handoff doesn't touch it).

## Goal

Bring the new `agents` domain (drives Claude Code CLI as an agent, Run launcher at
`/agents/run`, modeled on the external `agent-monitor` project's UX, built on Portal's own
shadcn-svelte/Tauri components) from "builds and type-checks" to "actually works when you click
through it." Original architect plan (P0–P6 phased) is approved; this session did P1 (core
`agents` domain: `AgentBackend` trait, `ClaudeCodeBackend`, `AgentRunService`, Run launcher,
live conversation view), a static-review fix pass on that, and a live-QA cycle on top. P2–P6
(persistence/Sessions list, Board/Kanban reuse, Analytics/Workflows, Codex/Cursor adapters) are
**not started**.

## State: **nothing committed this session.** All of the below is uncommitted working-tree changes.

The P1 `agents` domain code itself (Rust `src-tauri/portal-core/src/domains/agents/*`, Tauri
commands, frontend `src/lib/domains/agents/*` + `src/routes/agents/*`) was committed earlier by
a **different concurrent session** as part of merging unrelated Android remote-control work
(commit `c5de8161` on `main` — see the superseded handoff section in git history for why). Every
fix described below is **on top of that commit, uncommitted**.

### Review fix pass (first round) — done, gates green

Delegated to two `coder` subagents (backend Rust / frontend TS+Svelte), fixed all 14 findings
from a `crew:reviewer` pass plus QA-106:

- Backend: semaphore-permit leak in `service.rs` (never released on run end — would deadlock the
  11th concurrent run forever); split the `agents` RPC capability into `agents` (read-only,
  default-on) vs `agents.exec` (sensitive, added to `SENSITIVE_CAPS`, matches the
  `terminal`/`terminal.exec` pattern); `.no_window()` on the Windows spawn; `--setting-sources
""` to stop the per-run `--settings` hook merging with (and double-firing alongside) the
  user's real `~/.claude/settings.json`; `kill_on_drop(true)` + an RAII guard for the per-run
  temp settings file; a per-run UUID on the permission-hook route (`/hook/:token`); spawn now
  goes through `cmd /C claude` to match `coder/service.rs`'s existing resolution instead of a
  `which::which` path that could resolve to an unspawnable `.cmd` shim.
- Frontend: "End run"/"New run" buttons wired to `cancel()`/a new `reset()`; `pending` approval
  changed from a single overwritable slot to a list keyed by request id; `agentsApi.ts` moved
  from raw `invoke` to the repo's `invokeClient` convention; an event-backfill call added to
  close an early-event-drop race; a free-text model `Input` fallback when the backend's model
  list is empty; QA-106 (silent backend-list failure) now shows `PageError` + retry.
- **Deliberately left as-is**: the icon-import finding (`~icons/fa6-solid/folder-open`) — the
  frontend coder checked and `@lucide/svelte` isn't actually installed/used anywhere in this
  repo despite what `CLAUDE.md` says; the repo's real convention is `~icons/*`
  (unplugin-icons). **`CLAUDE.md`'s icon guidance is stale, not the code** — worth a doc fix
  separately, not done here.
- Gates after this round: `cargo check`/`cargo clippy`/`cargo test` (incl. the two capability
  guard tests) and `pnpm check`/`pnpm lint`/shadcn-check all clean, 0 errors.

### Live QA (first pass) — found 3 more issues, all 3 fixed and self-verified

A `qa-tester` subagent drove a real isolated second instance
(`node scripts/dev-instance.mjs 2`, vite `:1421` / CDP `:9223` / `src-tauri/target/instance-2` —
**left the owner's own instance-1 on `:1420`/`:9222` completely alone**, confirmed untouched at
the end) over Playwright/CDP against `/agents/run`. Results:

| #   | Check                                    | Result                                                                                |
| --- | ---------------------------------------- | ------------------------------------------------------------------------------------- |
| 1   | Approve/deny a tool call                 | blocker — [[QA-110]] — **now fixed**                                                  |
| 2   | Interrupt mid-run                        | pass                                                                                  |
| 3   | Cancel + New run                         | blocker — [[QA-111]] — **now fixed**                                                  |
| 4   | Bad cwd                                  | pass (clean toast, no hang/crash)                                                     |
| 5   | QA-106 regression                        | confirmed still fixed, live                                                           |
| 6   | Model fallback (empty list → text input) | pass                                                                                  |
| 7   | Concurrent approvals                     | not applicable yet — UI is still single-slot (unchanged by the QA-110 fix, see below) |

Filed: `.claude/qa/issues/QA-110-agents-run-approval-banner-buttons-disabled.md`,
`QA-111-agents-run-end-run-never-completes.md`,
`QA-112-agents-run-accept-and-remember-does-nothing.md` (needs-decision, minor — decided below).

**All three fixes landed and were self-verified live by the fixing coders** (screenshots +
repro-script reruns, not just gates):

1. **QA-111 — fixed.** Root cause turned out one level up from QA's own lead: not
   `process.rs`'s `Child` mutex, but `AgentRunService::cancel()` (`service.rs:189-193`) holding
   the **service-wide** `runs: Mutex<HashMap<...>>` for the full duration of `kill().await`
   (which used to block on `child.wait()`) — starving the per-run pump task of the same lock it
   needs to append the terminal `Exited` event, and (worse) stalling _every other run's_ event
   delivery while one was being killed. Fix: `process.rs`'s `kill_and_wait` → `start_kill` (sync,
   no longer awaits exit); `claude_code.rs`'s `ClaudeCodeRun::kill()` now signals the kill (lock
   held only for an instant) and immediately emits its own `AgentEvent::Exited` rather than
   waiting on the pump task's EOF-triggered race. Verified live: `qa-sweep2-cancel-raw.mjs`
   rerun unmodified, `Exited` now arrives at +0.5s (was stuck false after 20s), UI shows "Run
   ended" / "New run".
2. **QA-110 — fixed.** One-line root cause: `eventsToMessages.ts`'s reducer never cleared
   `running` while a `permission_request` was outstanding, and `ApprovalBanner`'s buttons are
   `disabled={busy}` where `CoderFeed` passes `busy={running}` (coder's own equivalent reducer
   already does this correctly). Fix: `if (pending.length > 0) running = false;` at the end of
   the reducer. Verified live: "Accept once" immediately clickable, a real Bash tool call
   approved and completed end-to-end.
3. **QA-112 — fixed as "hide the control."** No remember-rule concept exists anywhere in the
   Agents call chain or backend; decided to hide rather than build persistence (bigger, unscoped
   work). Added `showRemember?: boolean` (default `true`) to `ApprovalBanner.svelte`, forwarded
   through `CoderFeed.svelte`, Agents' `+page.svelte` passes `false` on both its main `CoderFeed`
   and the extra-approval-banner loop. Checked `coder`'s own usage doesn't pass it, so it still
   defaults `true` there — unaffected. Verified live: no remember button appeared across two
   approval rounds in a real run.

Gates re-run on the combined state after all three: `cargo check`/`cargo clippy`/`cargo test
rpc::` (8/8) and `pnpm check` (0 errors)/`pnpm lint` (eslint 0 errors, shadcn-check passed,
prettier clean on every file touched this session) — all clean.

**An independent `qa-tester` closed all three — status `verified` in their issue files, per this
repo's convention that closing is the qa-tester's job, not the fixer's:**

- **QA-110 verified**: Accept/Reject both resolve in <40ms (vs. the original 3s+ disabled-button
  timeout). Approve creates the expected file, reject doesn't; run recovers either way.
- **QA-111 verified**: single End-run now completes in ~300ms (was stuck forever). Concurrent
  check (run A cancelled via UI, run B via direct `invoke` on the same shared lock) confirmed B's
  events kept flowing through A's cancel — the one suspicious flat stretch in B's event count
  turned out to be the model's own turn pacing (a no-cancel baseline showed the identical shape),
  not a lock stall. **Stress test: 8 rounds × 3 simultaneous start+cancel (24 overlapping runs)
  — backend stayed responsive throughout, the previously-unreproduced crash lead did not recur.**
  Treating that lead as resolved by the lock fix, not filing anything further.
- **QA-112 verified**: no remember button/input renders on `/agents/run` across two live
  triggers; `coder`'s own `CoderFeed` usage (grepped) passes no `showRemember`, still defaults
  `true`, unaffected. One gap: the _second_ concurrent-approval banner's `showRemember={false}`
  was confirmed by reading the source, not by independently driving a live second concurrent
  approval (hard to force deterministically) — low risk, same prop, same gate as the first.

All three issue files now have a filled `## Verification` section. **This whole feature's QA
trail (QA-106 through QA-112) is fully closed.**

## What's left (in order)

1. **Decide + do the `coder/service.rs` refactor** (deferred from P1, not done): `coder`'s
   existing one-shot `advance_claude_cli`/`llm_provider = "claude-cli"` path should become a thin
   wrapper over the new `agents` service instead of its own separate CLI-spawn logic. Deferred
   originally because of risk to `coder`'s existing live permission behavior without budget to
   verify — still true, scope this carefully before doing it.
2. **Commit.** Nothing from this session (review-fix pass, QA-110/111/112 fixes) is committed.
   Decide commit granularity — probably: one commit for the review-fix pass, one for the
   QA-110/111/112 fixes, rather than one giant commit, but that's a call for whoever's doing it.
3. **Concurrent-approvals gap**: QA-112's second-banner `showRemember` path and QA's original
   checklist item #7 were both only confirmed via source read, not an independently-forced live
   second concurrent approval (hard to trigger deterministically) — low risk given it's the same
   prop/gate as the first banner, but worth a live confirmation if an easy way to force two
   concurrent subagent permission requests turns up.
4. Continue the phased roadmap: P2 (persistence + Sessions list/detail, a new migration), P3
   (reuse `KanbanBoard.svelte`/`taskUi.svelte.ts` for an agents board view — needs a
   `createTaskUiState({ columns })` overload, currently hardcoded to 3 columns — + Activity Feed
   - Dashboard), P4 (Analytics + Workflows), P5 (Codex adapter — the real test of whether the
     `AgentBackend` abstraction actually generalizes), P6 (Cursor adapter + the "also watch ambient
     sessions" decision the owner made, which nothing has implemented yet).

## Decisions made, don't re-litigate

- **Icon-import finding**: not a bug, `CLAUDE.md` is stale on this point. Don't re-flag it in a
  future review without also fixing the doc.
- **QA-112 ("Accept & remember")**: fixed by hiding the control, not by implementing real
  remember-rule persistence. If a future pass wants real persistence, that's new scope, not a
  reopened bug.
- **Paired remote devices CAN start agent runs** (owner's explicit call) — this is why
  `agents.exec`'s sensitivity is "requires explicit grant," not "blocked outright": the fix
  split the capability so it's possible to grant, not so it's impossible to use remotely.
- **Also watch ambient Claude/Codex sessions** (owner's explicit call, from the original
  planning round) — not implemented anywhere yet. Don't assume P1's Run-launcher-only scope was
  a silent downgrade of this; it's just not reached yet (P6-ish).
- Don't touch the owner's own already-running dev instance (`:1420`/`:9222`) when testing — use
  `node scripts/dev-instance.mjs 2` (or higher) for an isolated QA instance, same as this session
  and the one before it did.

## Files that matter

- `src-tauri/portal-core/src/domains/agents/{service.rs,claude_code.rs,process.rs}` — backend,
  review-fix pass + QA-111 fix (service-wide lock + `kill()` restructure), all done.
- `src-tauri/src/domains/agents/commands.rs`, `src-tauri/src/rpc.rs` — capability split
  (`agents` vs `agents.exec`), done.
- `src/lib/domains/agents/{api/agentsApi.ts,state/agentRun.svelte.ts,utils/eventsToMessages.ts,
components/RunLauncherForm.svelte}`, `src/routes/agents/run/+page.svelte` — frontend, review-fix
  pass + QA-110 fix (`running`/busy flag), all done.
- `src/lib/domains/coder/components/{ApprovalBanner.svelte,CoderFeed.svelte}` — got a new
  `showRemember` prop (default `true`, unaffected for `coder`'s own usage) for the QA-112 fix.
- `.claude/qa/issues/QA-106…QA-112*.md` — the issue trail for this specific feature, **all
  `verified`**. Fully closed.
- **Not this session's concern, don't touch without checking**: `src/app.css`,
  `src/lib/domains/devices/components/PairDeviceCard.svelte`, `src/routes/devices/+page.svelte`,
  `src/lib/utils/invokeClient.ts`, `.claude/handoff-devices-phone-qa.md`, and QA-109/113/114/115
  — a **different, still-active concurrent session** is working the devices/pairing domain in
  this same checkout. Don't attribute those diffs to this work, don't revert them, don't "clean
  them up" as unrelated noise when committing — check with whoever owns that session first.

## Things that bit this session

- The git history's own prior handoff (superseded by this one) already documents: `pnpm run
tauri dev` directly doesn't give you CDP (`:9222`) — use `dev-instance.mjs`; a bare
  `cargo check`/`cargo test` can leave generated TS bindings unprettified, fix with `pnpm
gen:types`, never hand-format; D: drive filling up is a recurring problem in this checkout
  (58GB free as of this session, `src-tauri/target/` alone is 38GB with a leftover
  `target/instance-2/` from an earlier QA attempt — safe to `cargo clean` instance-2 specifically
  if space gets tight again, don't touch the main `target/debug` the owner's instance-1 uses).
- Multiple other Claude sessions are active on this same account/checkout concurrently (confirmed
  via `ListAgents` — dozens of peer sessions, several on this exact repo). Always `git status`
  before assuming a file you didn't touch is "stray" — it's more likely another session's
  in-progress work.
