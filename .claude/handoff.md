# Handoff — Portal Desktop, coder engine work

Rewritten 2026-11-06 ~22:15. For someone with no memory of the session that produced it.
The previous version of this file described the state _before_ the coder testing was run;
that testing is now finished and its results live in
[`.claude/coder-test-plan.md`](./coder-test-plan.md).

## Goal

Two things the owner asked for, both now done:

1. Commit everything outstanding in this checkout.
2. Finish the remaining work in `.claude/coder-test-plan.md` — Parts A and B of the Coder
   engine test — then fix what it exposed.

## State: branch `feat/coder-cli-permission-mode`, nothing pushed

**Nothing is pushed in either repo** (portal_desktop or
`D:\projects\production\ai\agentic-ai\agent-platform`). The owner has not asked.

Nine commits, oldest first:

| commit    | what                                                                     |
| --------- | ------------------------------------------------------------------------ |
| `dd2d844` | pre-existing: let the Claude CLI engine run commands                     |
| `fb4be1e` | pre-existing: record file changes on the Claude CLI engine               |
| `8530cbb` | another session's tasks board / list / archive revamp (16 files)         |
| `e524574` | `.prettierignore` — ignore the generated `.impeccable/` hook cache       |
| `f53cd46` | QA board sweep (QA-076..082) + the then-current handoff and test plan    |
| `773bf2d` | Part A + Part B results written into the test plan; QA-083/084/085 filed |
| `e848a60` | fixes for QA-084 and QA-085                                              |
| `e2bb5a8` | fix for QA-083                                                           |
| `bfde40a` | repair of `e848a60`, which had shipped 4 type errors (see "What bit me") |

Working tree is clean. Gates, read with real exit codes: `pnpm check` 0 errors,
`pnpm lint` 0 errors, `npx vitest run src/lib/domains/coder/utils/feedBlocks.test.ts` 5/5.

## What was established (don't re-derive this)

**The coder is genuinely provider-independent.** It was tested end to end on Ollama
(`gemma4:e4b`) through `agent-platformd` on `http://127.0.0.1:18410`, workspace
`D:\projects\practice\todo-rn`. Streaming, the delegated tool loop, the Review approval
gate and the Changes panel all work, and the platform path populates `coder_file_changes`
**natively** — no transcript reconstruction. The Claude CLI path is the special case
(it owns its own tool loop, which is why `fb4be1e` had to rebuild changes from the
stream-json transcript).

Setup for Ollama needed **no bearer token, no model pull, no provider registration**:

- `gemma4:e4b` advertises `capabilities: [..., "tools", ...]`, so it does function calling.
  Only `ollama serve` had to be started.
- agent-platform already ships Ollama in its provider registry
  (`desktop/crates/server/src/llm_config.rs:99-122`, id `ollama`, base URL from
  `OLLAMA_API_BASE`, default `http://127.0.0.1:11434`). `provider_configured("ollama")` is
  true as soon as a base URL resolves — there is no registration endpoint and no DB row.
- Portal already lists **Ollama** in the coder engine dropdown (the only two entries are
  `Ollama` and `Claude Code (CLI)`) and posts `provider` + `model` per turn to
  `/api/v1/coder/chat/stream`.
- So the test plan's "issue a token, fill the AgentPlatform `api_key`" steps are **not
  needed**. They were also not attempted: reading `master.key` and trying it against the
  daemon is credential handling and the sandbox refused it, correctly. If a future run
  genuinely needs a token, the daemon reads `AGENT_PLATFORM_MASTER_KEY`
  (`desktop/crates/server/src/lib.rs:178`) and the desktop app writes
  `<app_dir>/master.key` (`desktop/crates/app/src/shell.rs:623-639`) — **ask the owner
  rather than reading it.**

What is provider-specific is **model capability, not plumbing**. `gemma4:e4b` called
`write_file` without the required `path` argument twice and only succeeded once the
arguments were dictated in the prompt. Portal's tool spec is correct
(`src-tauri/portal-core/src/domains/coder/tools.rs:28-39`). A third provider needs: an
entry in agent-platform's `PROVIDERS`, a reachable base URL, and a model whose tool calls
fill required args. Only the last is a judgement call.

## What is left

1. **QA-086 — four `role="button"` wrappers that nest real buttons.** The open item.
   `.claude/qa/issues/QA-086-role-button-wrappers-swallow-nested-buttons.md` has the
   sites:
   - `src/lib/domains/terminal/components/TabBar.svelte:186` (wraps a real
     `aria-label="Close tab"` button — worst of the four)
   - `src/lib/domains/ai/components/shared/AISessionCard.svelte:76`
   - `src/lib/domains/tasks/components/KanbanBoard.svelte:148,293`
   - `src/lib/domains/tasks/components/TaskList.svelte:61,181`

   The fix is the one already applied in `ChangesPanel.svelte` (see `e2bb5a8`): drop
   `role="button"`, keep `tabindex="0"`, add `role="group"` plus an explicit `aria-label`,
   and expose selection as `aria-current` rather than only a ring colour.

   **The two tasks files belong to another session** ("Tasks UI revamp and organization")
   which committed them as `8530cbb` in this same checkout. They were deliberately not
   edited to avoid a concurrent write; that session has been messaged with the locations
   and the fix. Check with it before touching them.
   `src/lib/domains/sdk/components/sidebar/SDKSidebarItem.svelte:60` also uses
   `role="button"` but nests nothing interactive — it is fine, leave it.

2. **Part C leftovers**, from the test plan:
   - the coder project dropdown shows a stale Linux path (`/media/tan/Workspace/...`) for
     the Portal Desktop project itself, on a Windows machine. Not investigated.
   - two unverified collapsed-sidebar observations (treat as "seen once", not findings).
   - **push** — the owner has not asked.

3. **Cosmetic:** several stale `pending` rows sit in `coder_file_changes` from test turns
   on old threads. `D:\projects\practice\todo-rn\App.tsx` is back at its baseline
   (`<Text style={styles.title}>Todos</Text>`), so nothing is actually dirty on disk.

## Files that matter

| path                                                       | why                                                        |
| ---------------------------------------------------------- | ---------------------------------------------------------- |
| `.claude/coder-test-plan.md`                               | the Part A / Part B results and the per-surface report     |
| `.claude/qa/issues/QA-083..086*.md`                        | 083/084/085 fixed + verified; **086 open**                 |
| `.claude/qa/scripts/app.mjs`                               | the CDP driver harness — use it, see below                 |
| `src/lib/domains/coder/utils/feedBlocks.ts`                | QA-084 fix: failed tool calls no longer counted as work    |
| `src/lib/domains/coder/utils/feedBlocks.test.ts`           | 5 cases covering that                                      |
| `src/lib/domains/coder/state/coderSession.svelte.ts`       | QA-085 fix: `llmConfigFor`, `syncThreadLlmConfig`          |
| `src/lib/domains/coder/components/ChangesPanel.svelte:380` | QA-083 fix, with a comment saying why it must stay a group |

## Decisions already made, and why

- **A thread keeps the engine it was created with.** Changing the engine dropdown no
  longer writes to the open thread; it sets the default for the next session and toasts to
  say so. Rewriting a live thread's `llm_provider` misattributed the messages and
  `coder_file_changes` rows the previous engine produced. Tradeoff accepted: the dropdown
  can now show an engine the current thread will not use, which is why the toast exists.
  Pinning or disabling the dropdown would be tidier but is a larger UI change.
- **Failed tool calls are reported, not hidden.** `Created a file (1 failed)` when some
  succeeded, `Failed creating 2 files` when none did. The all-failed wording reuses the
  existing in-progress phrasing as a gerund so no new per-tool vocabulary was needed.
- **QA-086 was filed rather than swept.** Fixing it means touching another session's
  freshly committed files, and the ask was QA-083 specifically.
- **`.claude/qa/shots/` was committed** (87 new PNGs, ~7 MB) because 510 shots were
  already tracked from earlier sessions — the precedent exists. The other session thought
  it was 53 MB of new history; it was not. If the owner wants shots gitignored going
  forward, that only pays off if the 510 already in history also go.

## What bit me — read this before you trust a gate

- **`pnpm check | tail -3` reports `tail`'s exit code, not the gate's.** I read "exit 0"
  twice from a failing gate and committed `e848a60` claiming "0 errors" while it shipped 4
  svelte-check errors. Always `pnpm check > /tmp/out.txt 2>&1; echo $?`.
- **I changed a method signature having seen only half its call sites**, because my grep
  output was truncated by a `head` and I didn't notice. `syncThreadLlmConfig` had four
  callers, not two. Count call sites before changing a signature.
- **`getByRole('button', { name: ... })` is unsafe in this app.** Several containers carry
  `role="button"` and their accessible name absorbs their children's text, so a role query
  resolves to the **wrapper**; the click lands on its focus handler and **silently does
  nothing** — no error, no console message. This produced a false "per-hunk reject is
  broken" finding that I nearly filed as a bug. Use
  `locator('button').filter({ hasText })` and assert `tagName === 'BUTTON'`. This is
  exactly what QA-086 is about.
- **Do not write a literal `<button>` in a Svelte comment** — the repo's
  `portal/no-raw-ui-elements` eslint rule matches it and fails `pnpm lint`.
- **Drive the app over CDP, not with pixel coordinates.** `.claude/qa/scripts/app.mjs`
  connects to `127.0.0.1:9222` with Playwright. Three traps: (a) its `open()` does a
  `page.goto`, which reloads the webview and **drops the active coder thread** — attach
  without navigating, or reselect the thread from the Sessions list first; (b) menu items
  are not uniformly `role="menuitem"` — find the open `[role="menu"]` and click its text,
  and press `Escape` between attempts because an open menu intercepts pointer events;
  (c) approval-banner buttons are **Accept once / Accept & remember / Reject**, not
  "Approve" — a regex looking for "Approve" reports no gate on a working gate.
  Importing the harness from outside the repo needs
  `createRequire('<repo>/package.json')('@playwright/test')`.
- Sessions-list rows show **truncated** prompt text — match a short prefix.
- In `Review` mode a file edit is **not** prompted; it lands and is reviewed in the Changes
  panel. The gate applies to **commands**. That is consistent with `Review` → `acceptEdits`.
- The board on `:18420` is **not** down — `GET /health` returns
  `{"name":"portal-desktop","ok":true,"version":"0.12.3"}` (the listener is
  `portal_desktop.exe` itself). `GET /api/projects` 404s, so an earlier "board API did not
  answer" report was a wrong path, not a dead endpoint.
- This checkout is **shared with another session** with no worktree. Never `git add -A`;
  stage by path. Stay off `src/lib/domains/tasks/**` and `.claude/qa/**` without checking.
