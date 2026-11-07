# Handoff — Portal Desktop, coder engine: model accuracy + the catalog/auth work

Rewritten 2026-11-06 ~23:55. For someone with **no memory** of the session that produced it.
The previous version of this file covered the coder engine testing itself (Parts A–C); that
is finished and its results live in [`.claude/coder-test-plan.md`](./coder-test-plan.md).
This version covers what came after: measuring the model, repairing tool-argument handling,
and fixing the coder's model dropdown.

## Goal

The owner asked, in order:

1. Whether the Ollama coder test was done properly and how accurate the coder is — can it
   be relied on for small projects or minor work? **Answered, with numbers. See Part D of
   the test plan.**
2. Fix tool-argument repair. **Done, `2ede7cf`.**
3. Fix the coder's model dropdown. **Done, `d61f4b8` + `e949648`.**
4. Delete the stale legacy settings file holding an old `agp_` token. **Done** (two files).

## State: branch `feat/coder-cli-permission-mode`, nothing pushed

**Nothing is pushed.** The owner has not asked. Five commits were added this session, on top
of the nine described in the previous handoff:

| commit    | what                                                                           |
| --------- | ------------------------------------------------------------------------------ |
| `2ede7cf` | salvage aliased tool arguments; repair-shaped error when one is missing        |
| `6a5ded4` | Part D of the test plan: measured model accuracy; QA-087 raised to major       |
| `d61f4b8` | report **which** check rejected a 401 from agent-platform (was always "token") |
| `e14fca2` | repair mangled paths in the QA-087 file (a `\a` escape ate a backslash)        |
| `e949648` | keep the model/engine selectors usable when the catalog is down                |

Gates, read with real exit codes: `cargo check` 0, workspace `cargo clippy --all-targets` 0,
`cargo test coder::` 66/66, `cargo test agent_platform` 2/2, `pnpm check` **0 errors**.

**`pnpm lint` fails, and not because of this work.** Prettier flags five files left
unformatted by the concurrent session: `CredentialVault.svelte`,
`routes/cloud/workloads/+page.svelte`, `.../workloads/pods/[pod]/+page.svelte`,
`.../workloads/statefulsets/+page.svelte`, `routes/sdk/versions/+page.svelte`. Because the
script is `prettier --check . && eslint src`, the `&&` means **eslint never runs** when
prettier fails — so "pnpm lint failed" tells you nothing about lint. Check your own files:
`npx prettier --check <file>` and `npx eslint src` (0 errors as of this writing). **Do not
run `pnpm format`** — it would rewrite the other session's in-progress files.

## What was established — do not re-derive any of this

### The coder works. The local models are the ceiling.

Measured, three tasks x two repetitions per model, driven against Portal's own tool specs
and system prompt, **gated on `tsc --noEmit`** (full method in Part D of the test plan):

| task                                            | gemma4:e4b | qwen3:8b |
| ----------------------------------------------- | ---------- | -------- |
| T1 — change one heading string                  | **2/2**    | **2/2**  |
| T2 — add a Text element showing a derived count | 1/2        | 1/2      |
| T3 — add a three-option filter row that filters | 0/2        | 0/2      |
| **total, compiling**                            | **3/6**    | **3/6**  |

So: exactly-specified single-line edits are reliable; "add an element that needs a style or
helper with it" is a coin flip; a feature in several coordinated parts never compiled.

**Never trust the turn summary.** Every failing run ended with the model stating it had made
the change, and two made no edit at all while reporting success. A compile or test step is
the only thing that separates those cases.

**The naive checker scored gemma4 6/6; four of those six compiled.** "The file changed" is
not a pass. This is the single most important lesson from the session.

### `qwen2.5-coder:7b` is unusable — do not reach for it

It scores 0/6. It never emits the `<tool_call>` wrapper its own Ollama template demands, so
every call arrives as assistant text and Portal's `strip_leaked_tool_syntax` drops it.
Reproduced with Portal's system prompt, a one-line system prompt, and none at all, so it is
the model, not Portal. `ollama show qwen2.5-coder:7b --template` declares tool support and
`/api/tags` reports `capabilities: [..., "tools", ...]` — **neither of those means the model
complies.** It is still installed if you want to retest after an Ollama update.

Installed and working: `gemma4:e4b` and `qwen3:8b`. The owner is currently set to
`gemma4:e4b` in both the provider default and the coder's own selection.

### Argument repair was worth doing but was never the ceiling

**Zero missing-argument failures in 24 tool calls across both working models.** The
`missing arg: path` loop that motivated `2ede7cf` reproduces at `temperature: 0` and did not
appear once at Portal's default temperature (Portal sets none). Keep the fix — it is cheap
and covers Claude- and OpenAI-shaped argument names — but do not expect it to move the
numbers above.

### The catalog/auth story, including a correction

The coder's model dropdown was dead because `GET /v1/catalog` returned 401 and the component
blanked both selector lists. Facts worth keeping:

- **Every** agent-platformd endpoint 401s without a bearer, including
  `/api/v1/coder/chat/stream`. Nothing is exempt. Verified by curl.
- The daemon distinguishes four cases behind that status, needing opposite fixes:
  `AUTH_REQUIRED` (no header), `TOKEN_INVALID` / `TOKEN_EXPIRED` (secret refused),
  `APP_ID_REQUIRED` / `APP_ID_UNKNOWN` (token fine, no AI entitlement). Portal used to
  collapse all four into "Invalid or missing agent-platform API token" — `d61f4b8` fixes
  that, and it is why this issue and an earlier session both spent their time on credentials.
- **What actually restored the auth was never proven.** `d61f4b8` only changed the error message.
  The provider config was re-saved (while switching models for Part D) before anyone
  recorded the before-state. If it recurs, the new message names the real code.
- **Correction to earlier advice in this session: do not swap the live key for the old
  `agp_` token.** Four settings copies existed; three held the same 52-char `agp_` token and
  only the live file held a 64-char hex secret — and the live file is the one that
  authenticates. So the `agp_` token is most likely what the daemon was _rejecting_ (plausibly
  issued by an older daemon instance). If the owner wants off a master-key-shaped secret,
  **issue a fresh `agp_` token**; do not reuse the deleted one.
- **All four stale settings copies are now deleted, with the owner's approval.** Two under
  `%APPDATA%/portal-desktop/` and two `ai-settings.json.bak-2026-11-06` files, all holding the
  same 52-char `agp_` token. Only `%APPDATA%/portal_desktop/ai-settings.json` remains, and the
  coder was verified working after each deletion. `portal-desktop/settings.json` was left in
  place deliberately — different file, and the live directory has a newer copy.
- The live key is 64 hex characters with no `agp_` prefix, which is master-key-shaped, while
  Portal's own Settings text says to paste a scoped `agp_` token and **not** the master key.
  Confirming it is literally the master key needs reading `master.key`, which the sandbox
  **denied as credential exploration** — correctly. Ask the owner; do not work around it.

### The coder's model is a localStorage pref, and that is by design

The model comes from `portal.ai.chatCatalog` in localStorage
(`src/lib/domains/ai/utils/chatCatalogPrefs.ts:1`), read by
`coderSession.initProviderDefaults`. Settings → AI → "Default model alias" writes the
provider row's `model`, which is only the fallback when no pref exists. That is not a bug.
The bug was that a dead catalog left the selectors empty so the pref could not be changed.

**A model picked in the dropdown persists correctly across reloads** — verified. An earlier
suspicion that it reverted was wrong: it came from writing localStorage directly while the
page was live, so the running page re-persisted its in-memory selection over the write. If
you need to change the model, **use the dropdown**, not localStorage.

## What is left

1. **QA-086 — four `role="button"` wrappers that nest real buttons.** Still open, unchanged
   from the previous handoff. `.claude/qa/issues/QA-086-role-button-wrappers-swallow-nested-buttons.md`
   has the sites: `terminal/components/TabBar.svelte:186` (worst — wraps a real
   `aria-label="Close tab"` button), `ai/components/shared/AISessionCard.svelte:76`,
   `tasks/components/KanbanBoard.svelte:148,293`, `tasks/components/TaskList.svelte:61,181`.
   The fix is the one already applied in `ChangesPanel.svelte` (`e2bb5a8`): drop
   `role="button"`, keep `tabindex="0"`, add `role="group"` plus an explicit `aria-label`,
   and expose selection as `aria-current`. **The two tasks files belong to the concurrent
   session** — check with it first. `sdk/components/sidebar/SDKSidebarItem.svelte:60` also
   uses `role="button"` but nests nothing interactive; leave it.

2. **Push.** Nothing is pushed in either repo. The owner has not asked.

3. **A latent hole deliberately not fixed** (speculative, no sighting): `app_paths::config_dir()`
   migrates the legacy config dir only when the new dir does **not exist at all**, and
   `copy_files_into` skips files that already exist. If the new directory gets created by
   some other feature first, legacy settings are never migrated and the config silently comes
   up empty. Only worth fixing if settings have ever been seen to vanish.

4. **Part C leftovers**, unchanged: the coder project dropdown shows a stale Linux path
   (`/media/tan/Workspace/...`) for Portal Desktop itself on a Windows machine; two
   unverified collapsed-sidebar observations (treat as "seen once"); several stale `pending`
   rows in `coder_file_changes` from test turns (cosmetic — files are at baseline on disk).

## Files that matter

| path                                                                        | why                                                               |
| --------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `.claude/coder-test-plan.md`                                                | Parts A–D. **Part D is the model accuracy measurement.**          |
| `.claude/qa/issues/QA-086-*.md`                                             | **open** — the role="button" wrappers                             |
| `.claude/qa/issues/QA-087-*.md`                                             | closed, with an explicit "what fixed it is unproven" section      |
| `src-tauri/portal-core/src/domains/coder/tools.rs`                          | `normalize_args`, `aliases`, `arg_error`, `required_args` + tests |
| `src-tauri/portal-core/src/domains/coder/service.rs:2461`                   | the **one** place tool args are normalized, before the gate       |
| `src-tauri/portal-core/src/domains/ai/providers/agent_platform_provider.rs` | `auth_failure_detail` + tests                                     |
| `src/lib/domains/ai/components/chat/ChatCatalogSelectors.svelte`            | the selector resilience fix (`e949648`)                           |
| `src/lib/domains/ai/utils/chatCatalogPrefs.ts`                              | `portal.ai.chatCatalog` — where the coder's model really lives    |
| `.claude/qa/scripts/app.mjs`                                                | the CDP driver harness — use it, see the traps below              |

## Decisions already made, and why

- **Tool arguments are normalized at the stream entry point, not inside `execute`.**
  `permissions.rs:39,94,102`, `summarize`, `suggested_rule` and `mutated_path` all read the
  canonical keys. Salvaging inside `execute` would gate the permission check on an empty
  path while writing the real one, and skip `record_change` — so the write would bypass the
  allowlist _and_ never appear in the Changes panel. One normalization upstream fixes all of
  them. Do not move it.
- **The required-argument list in the error is read back off `tool_specs()`**, not duplicated,
  so the message cannot drift from what the model was told.
- **Last-read-file inference for a missing `path` was deliberately not built.** Guessing a
  write target is how you overwrite the wrong file. Also skipped: nested-wrapper unwrapping
  (`{"args":{...}}`) and non-string coercion. Add only if a model you actually intend to use
  needs them.
- **The benchmark harness was not committed.** It lives in the session scratchpad; it is a
  measuring instrument, not shipped code. Part D documents the method well enough to rebuild
  it in ~150 lines if needed.
- **Model accuracy was measured by driving Ollama directly, not through the UI.** Portal's
  loop is already proven provider-independent (Part B), so the UI adds flakiness without
  adding signal. One end-to-end UI turn was run separately to prove the integration.

## What bit me — read this before you trust anything

- **A non-empty diff is not a pass, and the agent's own summary is worthless.** Gate on
  `tsc --noEmit` / tests. This cost a wrong 6/6 score that had to be retracted.
- **`pnpm lint` failing says nothing about your code** — see the `&&` short-circuit above.
- **A model's advertised `capabilities` and its template's tool support do not mean it
  emits tool calls.** `qwen2.5-coder:7b` declares both and emits none.
- **Driving the app over CDP:** `open()` does a `page.goto`, and **the app frequently
  redirects elsewhere** (`/projects/1`, `/settings/general`) — always loop:
  `if (!p.url().includes('/ai/coder')) { await p.goto(...); }` up to ~5 times, or every later
  locator fails for the wrong reason.
- **The composer sends on Ctrl/Cmd+Enter, not Enter** — plain Enter inserts a newline, and
  the placeholder warns that 2+ lines auto-spawns parallel agents. The send control is
  `button[title^="Send"]`. Prefer typing with `.type()` over `.fill()`.
- **An open menu intercepts pointer events** — press `Escape` between attempts, and expect
  "`<html>` intercepts pointer events" when something modal is open.
- **For dropdown options, `p.getByText(/value/).first()` works where
  `[role="dialog"] *`/`[role="option"]` filters match nothing.**
- **`getByRole('button', { name })` is unsafe in this app** — containers carry
  `role="button"` and absorb their children's accessible name, so the click lands on the
  wrapper and silently does nothing. Use `locator('button').filter({ hasText })` and assert
  `tagName`. That is what QA-086 is about.
- **Do not set the coder's model by writing localStorage while the app is running** — the
  live page re-persists its in-memory selection over you, and it looks like a persistence
  bug. Use the dropdown.
- **If you point the platform base URL at a dead port to test a failure path, restore it in a
  `finally`.** A crashed script left the owner's app talking to nothing until it was repaired
  by editing `ai-settings.json` directly (base_url only).
- **Editing a settings file leaves a `.bak` holding a copy of the api_key.** Delete it.
- **Reading `master.key` is denied as credential exploration.** That is correct — ask the
  owner instead of working around it.
- **Do not use `\a`, `\p` etc. in a non-raw Python heredoc that writes Markdown** — `\a`
  became a BEL character mid-path and mangled three file paths (`e14fca2` repaired it).
- **This checkout is shared with the concurrent session "Tasks UI revamp and organization"**
  and has no worktree. **Never `git add -A`; stage by path.** Stay off
  `src/lib/domains/tasks/**` and the five prettier-flagged files without checking. A Rust
  edit triggers a dev rebuild + relaunch (~45–75s) which kills their CDP runs.
- `.claude/qa/issues/` now also contains a `UX-*` series from another session. Leave it.

## Starting fresh

Open a session in `D:\projects\production\portal\portal_desktop` and say:

> read .claude/handoff.md and .claude/coder-test-plan.md
