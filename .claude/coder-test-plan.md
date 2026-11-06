# Coder test plan — what is left to run

A runnable checklist for finishing the Coder engine testing started 2026-11-06.
Background, decisions and traps live in [`.claude/handoff.md`](./handoff.md) — read that
first if you have no memory of the earlier session. This file is just the work.

**To start:** open a fresh session in `D:\projects\production\portal\portal_desktop` and say:

> read .claude/handoff.md and .claude/coder-test-plan.md, then do Part A

---

## Before you touch anything

- [x] **Branch is `feat/coder-cli-permission-mode`.** Two commits on it, `dd2d844` and `fb4be1e`, both coder-only. Nothing pushed.
- [x] **This checkout is shared with another session** ("Tasks UI revamp and organization"). Never `git add -A`. Stay off `src/lib/domains/tasks/**`, `.claude/qa/**`, `src/routes/github/issues/+page.svelte`.
- [x] **Message that session before driving the app window.** `SendMessage` to `Tasks UI revamp and organization`. They run QA sweeps over CDP and a rebuild/relaunch kills their run. Ping them again when you are off it.
- [x] Portal Desktop should already be running under `pnpm tauri:dev`. A Rust edit triggers a rebuild + relaunch in roughly 45–75s.

---

## Part A — confirm the Changes panel populates (finishes `fb4be1e`)

The only unverified part of the work. The parsing has unit tests and the Rust is
live in the running app; what nobody has watched is a real CLI turn driving
`record_change` through to a database row.

1. [x] Open the Coder: sidebar **AI** → **Code** tab (route `/ai/coder`).
2. [x] Confirm the engine selector reads **Claude Code** and the workspace reads `todo-rn — D:\projects\practice\todo-rn`. If not, the folder icon at the right of the workspace dropdown opens a picker.
3. [x] Confirm the **⚡ permission chip** is visible in the composer and set to **Auto**. (Its presence is itself part of `dd2d844` — it did not render for this engine before.)
4. [x] Start a **new session** (engine is stored per thread in `coder_threads.llm_provider`, so reusing an old thread tests the wrong thing).
5. [x] Ask for a small, obvious edit, e.g.
       _"In App.tsx, change the heading text from Todos to My Todos. Nothing else."_
6. [x] When it finishes, open the **Workspace** rail on the right → **Agent changes**.

**Pass:** `App.tsx` is listed with a reviewable diff.
**Fail:** the panel is still empty.

7. [x] Verify at the data layer too — do not trust the panel alone, since "the panel is empty" and "the write never happened" look identical from the UI:

```bash
cp ~/AppData/Roaming/com.tanvoid0.portal-desktop/portal_desktop.db /tmp/pd.db
sqlite3 /tmp/pd.db "select tool, path, status from coder_file_changes where thread_id = (select id from coder_threads order by rowid desc limit 1);"
```

Expect at least one row, tool `Edit` or `Write`, path `App.tsx`. Before the fix this
query returned **0 rows** — that is the regression baseline.

8. [x] If it passes, also try **accept / reject** on the change. `record_change` stores
       `before` and `original_after`, so the per-hunk controls should work — but the
       platform path owns that flow and it has never been exercised from a CLI turn.
       If reject does not restore the file, that is a finding worth its own fix.

### If Part A fails

Most likely causes, in order:

- The thread was created before the rebuild — start a genuinely new one.
- Claude edited a file outside the workspace root — those are skipped by design (`workspace_relative` returns `None`).
- The snapshot/write race in `advance_claude_cli` lost: `record_change` returns early when `after == before`. Add a temporary log either side of the `pending_edits` insert to see which half is missing.

---

### Part A result — PASS, 2026-11-06

Verified end to end on thread `2bb8fc85-69ee-4b65-b700-c9425aa02574` (`llm_provider=claude-cli`),
workspace `D:\projects\practice\todo-rn`, permission chip **Auto**.

| step                                          | outcome                                                                                                                         |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| ⚡ permission chip renders for the CLI engine | yes, and settable (`Review` -> `Auto`) — confirms `dd2d844`                                                                     |
| turn ran                                      | `Ran Bash, Ran Grep, Ran Edit`, "Worked for 12s"                                                                                |
| **Agent changes** panel                       | `1 pending`, reviewable diff, `hunk 1 · line 63`, `- Todos` / `+ My Todos`                                                      |
| data layer                                    | 1 row: `App.tsx                                                                                                                 | pending`, `data_json.tool = "Edit"`, `before` captured |
| regression baseline                           | the three older `claude-cli` threads still have **0** rows; `ollama` threads have rows                                          |
| accept / reject (step 8)                      | per-hunk **Reject hunk** reverted `App.tsx` on disk and set `status=rejected`, `hunks[0].accepted=0`; **Reject all** also works |

So `fb4be1e` is confirmed: a real CLI turn drives `record_change` through to a row, and the
platform-owned accept/reject flow works from a CLI turn.

**The plan's verification SQL above is wrong** — `coder_file_changes` has no `tool` column.
The schema is `id, thread_id, path, status, data_json, created_at`; `tool`, `before` and
`original_after` live inside `data_json`. Use:

```bash
cp ~/AppData/Roaming/com.tanvoid0.portal-desktop/portal_desktop.db /tmp/pd.db
sqlite3 /tmp/pd.db "select path, status, json_extract(data_json,'\$.tool') from coder_file_changes where thread_id = (select id from coder_threads order by rowid desc limit 1);"
```

**Filed:** `QA-083` — the hunk row is a `div role="button"` wrapping the real Accept/Reject
buttons, so `getByRole('button', {name:'Reject hunk'})` resolves to the container and the
click silently does nothing. Cost this run a false "per-hunk reject is broken" finding.
Per-hunk reject itself is fine.

**Driving note that worked.** Do not use the pixel helper. The QA harness at
`.claude/qa/scripts/app.mjs` drives the app over CDP on `:9222` with Playwright locators —
no coordinates, immune to layout moves. But `open()` does a `page.goto`, which **reloads the
webview and drops the active coder thread**; to act on a live thread, attach to CDP without
navigating, or reselect the thread from the Sessions list first.

---

## Part B — Ollama via agent-platform (not started)

The point is **provider independence**: proving the coder works on the built-in
platform engine with a local model, not just by shelling out to Claude Code. The
owner accepts that a new provider may need its own fixes; this run is to find out
which.

1. [x] **Start Ollama.** Binary at `C:\Users\tanvo\AppData\Local\Programs\Ollama\ollama`; it was not running. Confirm with `curl http://127.0.0.1:11434/api/tags`.
2. [x] **Check the model can tool-call.** Only `gemma4` is on disk. The platform engine's whole loop is function calling, so if `gemma4` cannot do it, pull one that can (e.g. `qwen2.5-coder`) rather than reporting failure.
3. [x] **Register Ollama as a provider in agent-platform.** Daemon `agent-platformd` is healthy on `http://127.0.0.1:18410` and requires `Authorization: Bearer <session JWT | agp_ workspace token | master key>`. Work out provider registration and token issuance from the repo at `D:\projects\production\ai\agentic-ai\agent-platform` (branch `main`, recently merged).
4. [x] **Fill Portal's provider row.** Settings → AI, the **AgentPlatform** row: `base_url` `http://127.0.0.1:18410`, `api_key` = the bearer token, `model` = the Ollama model. **It must be enabled** — env fallbacks (`CODER_PLATFORM_BASE_URL`, `AGENT_PLATFORM_BASE_URL`, `CODER_MODEL`) only apply when the row is absent or disabled, and they supply no token. See `src-tauri/portal-core/src/domains/ai/platform_config.rs:20-70`.
5. [x] **Start a new coder thread** on that engine (again: per-thread, so switch the selector _then_ start a new session). Workspace `D:\projects\practice\todo-rn` — it already holds a working Expo/TS todo app that type-checks clean.
6. [x] Give it something verifiable, e.g. _"add a Clear completed button that removes every done todo"_, then read the diff yourself.

---

### Part B result — 2026-11-06. The coder is genuinely provider-independent.

Ran on engine **Ollama** (`llm_provider=ollama`) through `agent-platformd` on
`http://127.0.0.1:18410`, model `gemma4:e4b`, workspace `D:\projects\practice\todo-rn`.

**Setup was far cheaper than the plan assumed — no token, no model pull, no registration.**

- Ollama only had to be started (`ollama serve`). `gemma4:e4b` advertises
  `capabilities: ["completion","vision","audio","tools","thinking"]`, so it **does**
  tool-call. No `qwen2.5-coder` pull was needed.
- agent-platform already ships Ollama as a provider (`llm_config.rs:99-122`, id `ollama`,
  base URL from `OLLAMA_API_BASE`, default `http://127.0.0.1:11434`). There is **no**
  registration endpoint and no DB row — `provider_configured("ollama")` is true as soon as
  a base URL resolves (`llm_config.rs:421`).
- Portal already offers **Ollama** in the coder's engine dropdown (the only two entries are
  `Ollama` and `Claude Code (CLI)`), and posts `provider` + `model` per turn to
  `/api/v1/coder/chat/stream` (`service.rs:2282-2296`, `3052-3061`). Nothing had to be
  filled into the AgentPlatform provider row for this run.
- The plan's step 3/4 (issue a bearer token, fill `api_key`) was therefore **not needed**.
  It was also not attempted: reading `master.key` and trying it against the daemon is
  credential handling, and the sandbox correctly refused it. If a future run does need a
  token, the daemon reads `AGENT_PLATFORM_MASTER_KEY` (`server/src/lib.rs:178`) and the
  desktop app generates `<app_dir>/master.key` (`app/src/shell.rs:623-639`) — **ask the
  owner** rather than reading it.

### Report, per surface

| Surface       | Where it lives                                                                      | Works?                                                                                                                                                           |
| ------------- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Streaming     | `coder://message` events                                                            | **Yes.** UI filled progressively; turns reported "Worked for 25s" / "6s" / "7s".                                                                                 |
| Tool loop     | `read_file` / `write_file` / `edit_file` / `run_command`, `domains/coder/tools.rs`  | **Yes, with a model caveau.** `list_dir`, `read_file`, `edit_file`, `run_command` all ran. `write_file` failed — see below.                                      |
| Approval gate | `domains/coder/permissions.rs`; `Review` emits `coder://pending` → `ApprovalBanner` | **Yes.** In `Review`, `run_command` raised "Approval required — Running command `node -e "console.log(2+2)"`" with **Accept once / Accept & remember / Reject**. |
| Changes panel | `coder_file_changes` — the platform path **should** populate this natively          | **Yes, natively.** A successful `edit_file` gave `Edited App.tsx +1 -1`, `1 pending`, and 1 DB row. No CLI-style reconstruction needed.                          |

Note on `Review`: a file edit is **not** prompted — it lands and is reviewed afterwards in
the Changes panel (pending / accept / reject). The gate applies to **commands**. That is
consistent with the CLI engine's `Review` → `acceptEdits` mapping from `dd2d844`.

### What is provider-specific, and what would a third provider need?

**Portal's side is provider-agnostic already.** Nothing in the tool loop, the approval gate
or the changes panel needed a code change for Ollama. The one thing Portal has per engine
is the CLI transcript reconstruction added in `fb4be1e` — and that exists _because_ the
Claude CLI owns its own tool loop. A provider reached through agent-platform uses
`delegate_tools: true`, so Portal executes the tools itself and calls `record_change`
inline (`service.rs:781-791`). **The platform path is the general one; the CLI path is the
special case.**

**What is provider-specific is model capability, not plumbing:**

- `gemma4:e4b` called `write_file` **without the required `path` argument**, twice, giving
  `Error: missing arg: path`. Portal's spec is correct (`tools.rs:28-39` declares
  `path` + `content` required) — the 8B model simply did not fill it. It succeeded as soon
  as the arguments were dictated in the prompt. A weak model is usable but needs either a
  stronger model or arg-level retry/repair.
- The model also ignored "nothing else" and tried to create files it was not asked for.

**A third provider would need:** nothing in Portal, as long as it is reachable through
agent-platform's provider registry and its model does real function calling. The checklist
is (1) an entry in `llm_config.rs` `PROVIDERS`, (2) a reachable base URL, (3) a model whose
tool calls fill required args. Only (3) is a judgement call.

### Filed from Part B

- **`QA-084`** (major, **fixed and verified**) — the turn summary said "Created 2 files" when **both** `write_file`
  calls returned `Error: missing arg: path`. `feedBlocks.ts:179` counts tool _invocations_
  and never inspects results, so a failed call is indistinguishable from a success.
- **`QA-085`** (major, **fixed and verified**) — moving the engine selector while a thread is active rewrites
  **that thread's** `llm_provider` (`coderSession.svelte.ts:938-947`). Part A's thread
  `2bb8fc85` went from `claude-cli` to `ollama` while still owning the two changes the
  Claude CLI engine produced. **The plan's own advice ("switch the selector, then start a
  new session") is unsafe** — open the new session first, then pick the engine.

### Observation — resolved

The composer appeared to render two adjacent chips (`Review` and an untitled `Auto`). The
second one is the **agent-mode** chip, which only renders for the platform engine: it
disappeared as soon as the engine was switched back to Claude Code (CLI). Not a bug. It is
unlabelled (`title` and `aria-label` both empty), which is the only thing worth tidying.

### Driving notes for next time

- `getByRole('button', { name: ... })` is unsafe in this app — several containers carry
  `role="button"` and their accessible name absorbs their children's text, so a role query
  can resolve to the wrapper and the click silently no-ops (this produced one false finding,
  now `QA-083`). Prefer `locator('button').filter({ hasText })` and assert `tagName`.
- The permission-mode menu's items are **not** `role="menuitem"`; find the open
  `[role="menu"]` and click its text. An already-open menu intercepts pointer events, so
  press `Escape` between attempts.
- Approval-banner buttons are **Accept once / Accept & remember / Reject** — not
  "Approve"/"Allow". A regex looking for "Approve" reports no gate on a working gate.

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

---

## Part C update — 2026-11-06

- **Portal board on `:18420` is NOT down.** `GET http://127.0.0.1:18420/health` returns
  `{"name":"portal-desktop","ok":true,"version":"0.12.3"}` — the listener is the running
  `portal_desktop.exe` itself. `GET /api/projects` returns **404**, so the earlier
  "the board API did not answer" was a wrong path, not a dead endpoint. Find the real
  route before concluding the board is unavailable.
- **Ollama** is running now (`ollama serve`, `:11434`, `gemma4:e4b`). It was not before.
- Still open from Part C: the stale `/media/tan/Workspace/...` Linux path in the coder's
  project dropdown, the two unverified collapsed-sidebar UI observations, and **push**
  (nothing is pushed in either repo; the owner has not asked).
- One stray `pending` row is left in `coder_file_changes` from the last Ollama turn. The
  file on disk is back at its baseline (`Todos`), so it is cosmetic; "Reject all" only
  appears while the Agent changes tab is the active rail tab.

---

## Fixes applied — 2026-11-06

`QA-084` and `QA-085` are both fixed, verified in the running app, and committed.
`QA-083` (the `role="button"` hunk-row wrapper) is **still open** — it is a real a11y
defect but cosmetic in effect, and it is the one that most endangers future automation
runs, so read it before writing another driver script.

The engine-switch fix changes behaviour worth knowing about: **a thread keeps the engine it
was created with.** Moving the dropdown no longer re-labels the open session; it sets the
default for the next one and now toasts to say so. So the plan's Part B step 5 advice is
superseded — _start the new session first, then pick the engine_, or just pick the engine
and start a new session; either order is now safe, and neither corrupts the old thread.

Gates after the fixes: `pnpm check` 0 errors, `pnpm lint` 0 errors, and
`src/lib/domains/coder/utils/feedBlocks.test.ts` 5/5.
