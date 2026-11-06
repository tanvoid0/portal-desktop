# Coder test plan — what is left to run

A runnable checklist for finishing the Coder engine testing started 2026-11-06.
Background, decisions and traps live in [`.claude/handoff.md`](./handoff.md) — read that
first if you have no memory of the earlier session. This file is just the work.

**To start:** open a fresh session in `D:\projects\production\portal\portal_desktop` and say:

> read .claude/handoff.md and .claude/coder-test-plan.md, then do Part A

---

## Before you touch anything

- [ ] **Branch is `feat/coder-cli-permission-mode`.** Two commits on it, `dd2d844` and `fb4be1e`, both coder-only. Nothing pushed.
- [ ] **This checkout is shared with another session** ("Tasks UI revamp and organization"). Never `git add -A`. Stay off `src/lib/domains/tasks/**`, `.claude/qa/**`, `src/routes/github/issues/+page.svelte`.
- [ ] **Message that session before driving the app window.** `SendMessage` to `Tasks UI revamp and organization`. They run QA sweeps over CDP and a rebuild/relaunch kills their run. Ping them again when you are off it.
- [ ] Portal Desktop should already be running under `pnpm tauri:dev`. A Rust edit triggers a rebuild + relaunch in roughly 45–75s.

---

## Part A — confirm the Changes panel populates (finishes `fb4be1e`)

The only unverified part of the work. The parsing has unit tests and the Rust is
live in the running app; what nobody has watched is a real CLI turn driving
`record_change` through to a database row.

1. [ ] Open the Coder: sidebar **AI** → **Code** tab (route `/ai/coder`).
2. [ ] Confirm the engine selector reads **Claude Code** and the workspace reads `todo-rn — D:\projects\practice\todo-rn`. If not, the folder icon at the right of the workspace dropdown opens a picker.
3. [ ] Confirm the **⚡ permission chip** is visible in the composer and set to **Auto**. (Its presence is itself part of `dd2d844` — it did not render for this engine before.)
4. [ ] Start a **new session** (engine is stored per thread in `coder_threads.llm_provider`, so reusing an old thread tests the wrong thing).
5. [ ] Ask for a small, obvious edit, e.g.
       _"In App.tsx, change the heading text from Todos to My Todos. Nothing else."_
6. [ ] When it finishes, open the **Workspace** rail on the right → **Agent changes**.

**Pass:** `App.tsx` is listed with a reviewable diff.
**Fail:** the panel is still empty.

7. [ ] Verify at the data layer too — do not trust the panel alone, since "the panel is empty" and "the write never happened" look identical from the UI:

```bash
cp ~/AppData/Roaming/com.tanvoid0.portal-desktop/portal_desktop.db /tmp/pd.db
sqlite3 /tmp/pd.db "select tool, path, status from coder_file_changes where thread_id = (select id from coder_threads order by rowid desc limit 1);"
```

Expect at least one row, tool `Edit` or `Write`, path `App.tsx`. Before the fix this
query returned **0 rows** — that is the regression baseline.

8. [ ] If it passes, also try **accept / reject** on the change. `record_change` stores
       `before` and `original_after`, so the per-hunk controls should work — but the
       platform path owns that flow and it has never been exercised from a CLI turn.
       If reject does not restore the file, that is a finding worth its own fix.

### If Part A fails

Most likely causes, in order:

- The thread was created before the rebuild — start a genuinely new one.
- Claude edited a file outside the workspace root — those are skipped by design (`workspace_relative` returns `None`).
- The snapshot/write race in `advance_claude_cli` lost: `record_change` returns early when `after == before`. Add a temporary log either side of the `pending_edits` insert to see which half is missing.

---

## Part B — Ollama via agent-platform (not started)

The point is **provider independence**: proving the coder works on the built-in
platform engine with a local model, not just by shelling out to Claude Code. The
owner accepts that a new provider may need its own fixes; this run is to find out
which.

1. [ ] **Start Ollama.** Binary at `C:\Users\tanvo\AppData\Local\Programs\Ollama\ollama`; it was not running. Confirm with `curl http://127.0.0.1:11434/api/tags`.
2. [ ] **Check the model can tool-call.** Only `gemma4` is on disk. The platform engine's whole loop is function calling, so if `gemma4` cannot do it, pull one that can (e.g. `qwen2.5-coder`) rather than reporting failure.
3. [ ] **Register Ollama as a provider in agent-platform.** Daemon `agent-platformd` is healthy on `http://127.0.0.1:18410` and requires `Authorization: Bearer <session JWT | agp_ workspace token | master key>`. Work out provider registration and token issuance from the repo at `D:\projects\production\ai\agentic-ai\agent-platform` (branch `main`, recently merged).
4. [ ] **Fill Portal's provider row.** Settings → AI, the **AgentPlatform** row: `base_url` `http://127.0.0.1:18410`, `api_key` = the bearer token, `model` = the Ollama model. **It must be enabled** — env fallbacks (`CODER_PLATFORM_BASE_URL`, `AGENT_PLATFORM_BASE_URL`, `CODER_MODEL`) only apply when the row is absent or disabled, and they supply no token. See `src-tauri/portal-core/src/domains/ai/platform_config.rs:20-70`.
5. [ ] **Start a new coder thread** on that engine (again: per-thread, so switch the selector _then_ start a new session). Workspace `D:\projects\practice\todo-rn` — it already holds a working Expo/TS todo app that type-checks clean.
6. [ ] Give it something verifiable, e.g. _"add a Clear completed button that removes every done todo"_, then read the diff yourself.

### Report, per surface

| Surface       | Where it lives                                                                      | Works? |
| ------------- | ----------------------------------------------------------------------------------- | ------ |
| Streaming     | `coder://message` events                                                            |        |
| Tool loop     | `read_file` / `write_file` / `edit_file` / `run_command`, `domains/coder/tools.rs`  |        |
| Approval gate | `domains/coder/permissions.rs`; `Review` emits `coder://pending` → `ApprovalBanner` |        |
| Changes panel | `coder_file_changes` — the platform path **should** populate this natively          |        |

Finish with: **what here is provider-specific, and what would a third provider need?**

---

## Part C — loose ends, lower priority

- [ ] **Stale workspace path.** The coder's project dropdown showed `/media/tan/Workspace/...` — a Linux path — for the Portal Desktop project itself, on a Windows machine. Not investigated.
- [ ] **Two UI observations, unverified**, passed on by the other session's layout work. From AI → Home with the sidebar collapsed, a click around (235, 114) in window coords navigated to `/tasks`; and the collapsed icon rail scrolls, so bottom icons (AI, Settings) can sit partly out of view. Both may be stale coordinates rather than real hit-area bugs — treat as "seen once", not as findings.
- [ ] **Push.** Nothing is pushed in either repo. The owner has not asked yet.
- [ ] **Portal board.** These items could not be filed as tasks: the board API did not answer on `http://127.0.0.1:18420` even with the desktop app running. Worth a look on its own — if the board is meant to be the orchestration surface, that endpoint being down is the thing blocking it.

---

## Driving the app

It is a Tauri window, not a page — drive it as a window. Helper at
`C:/Users/tanvo/AppData/Local/Temp/claude/6ce5694c4c5839e1/5c08034c-1f05-47a7-a84a-86a304b95e74/scratchpad/pd.sh`
(`shot` / `click` / `type` / `key` / `scroll` / `title`), but it is in a session-scoped
temp dir and **may be gone** — it wraps `C:/Users/tanvo/.claude/plugins/cache/tanvoid0/hands/0.5.1/win.ps1`,
see that plugin's `stacks/tauri.md`.

- Coordinates are **window** coordinates, title bar included.
- `title` reads the document title — the cheapest navigation assertion available.
- **Coordinates go stale when the layout changes.** This is what defeated the last attempt at Part A: a tab strip moved and clicks started routing elsewhere. Re-screenshot after any UI change rather than reusing a coordinate from earlier, and assert with `title` after every navigation click.
- Posted clicks land intermittently while the webview is busy — loop with a `title` check rather than clicking once and assuming.
- Screenshots are expensive and cannot be evicted from context. Delegate "look at this and tell me what it says" to a subagent.
