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

---

# Handoff 2 — UX density sweep, wave 3 (appended 2026-11-07)

Separate workstream from everything above; that section is about the coder engine and is
finished. This one is the UX board. **Do not merge the two — they share only the branch.**

## Goal

The owner's complaint: every page is too spacious for small screens, the title bars are too
tall, space is wasted, and no page uses a right sidebar where one would help. A review sweep
measured the app at 1280x800 and 1024x600 and filed 20 findings at `.claude/qa/issues/`;
`UX-000-index.md` ranks them by win / effort. Waves 1 and 2 compacted the shared shell and
rolled 32 pages onto it. Wave 3, below, did the thirteen findings that were left.

## Done

Eleven fixer agents, then two independent testers. **Both gates green: `pnpm check` 0 errors
across 3814 files, `npx eslint src --quiet` 0 errors.**

Verified by a tester that did not write the fix: **UX-005** (settings nav — 14/14 sections,
no scroll at 1280x800, hardcoded `Configured: 4/7` gone), **UX-006** (`/logs` 9 -> 29 lines,
rows 44 -> 24px), **UX-007** (`/automation/blocks` chrome 278 -> 186px, cards ~300 -> 126px),
**UX-014** (`/sdk` tabs deleted, `/sdk/manager` -> `/sdk/managers`, all redirects walked),
**UX-016** (7 form routes on PageHeader), **UX-019** (already fixed upstream in `d0fccf8`),
**QA-088**. **UX-013** verified by code read only — cloud went 25 -> 18 route files and
1875 -> 579 lines, with every redirect proved using SvelteKit's own `create_manifest_data` +
`parse_route_id` over the real route tree (script: `.claude/qa/scripts/UX-013-routes.mjs`).

Splits (UX-018): `tasks/[id]` 965 -> 16, `scripts/[id]` 1031 -> 292, `IdeSettings` 1187 -> 47,
`SDKSidebar` 959 -> 535 (−584/+165 — deletion, not relocation), `BlocksPage` 1088 -> 319.
`/scripts/new` lost 246 lines to a shared `ScriptForm`. Deleted outright:
`FrameworkIdeSettings.svelte` (284 lines, zero references), `WorkloadTabs.svelte`, four
pods-only cloud components.

## Left

**1. Five findings are fixed but NOT verified — nobody has opened the app since.** Every
number in them is a fixer's arithmetic: **UX-008** `/projects` (claims 11,731px -> ~1,980px
collapsed, 190 -> 39 buttons), **UX-009** `/projects/audit` (1 -> 8 rows at 1280, 0 -> ~4 at
1024), **UX-010** `/pipeline-runs` (sticky rail, `?from=` gone), **UX-011** `/projects/[id]`
(chrome 192 -> 102px), **UX-012** `/tasks` (three columns at 1280, `?task=` rail). Also
untested: `/scripts/[id]`, and the SDK sidebar's mobile sheet below 768px.

**2. Six regressions this wave introduced**, each filed: `QA-098` major (SDK sidebar tree
opens fully collapsed, never expands the selected SDK's category — both `$effect`s defeated
by `lastSelectedItemId` being set on the first empty pass), `QA-099` polish (sidebar header
subtitle cut in half), `QA-101` major (block card titles truncate to one letter at 1024),
`QA-102` major (invisible hover overlay swallows framework-chip clicks), `QA-090` minor
(`/logs` has no way back to the live tail), `QA-091` minor (settings sub-items lack
`aria-current`).

**3. Pre-existing, found while testing — do not bill these to the wave:** `QA-097` **blocker**
(`/sdk` hangs on "Loading SDK data..." on a _direct_ load; UX-014's own original screenshot
already showed this), `QA-096`, `QA-100`, `QA-092`, `QA-093`, `QA-094`, `QA-095`, `QA-103`,
`QA-089` (fixed at its one call site, but the trap is still in the hook —
`UseResourceActionsOptions.resources` still accepts a getter and still freezes it silently).

**4. `UX-020`**, which I filed: `--sidebar-width-domain` is 20rem, sized for the two-line nav
items this wave deleted, so pages have begun pinning it inline one at a time. It is a shared
token in `src/app.css` and wants one deliberate pass across all four domain sidebars, not a
page at a time.

## Files that matter

- `.claude/qa/issues/` — the board. `UX-000-index.md` first.
- Shared notes, fuller version of this with per-fixer detail:
  `C:\Users\tanvo\AppData\Local\Claude-Data\claude_pzs8sxrjxfjjc\side-session-notes\local_d12dc36e-9cce-401e-9132-361b466b7ab9\ux-board-wave3-handover.md`
- New this wave: `src/lib/domains/cloud/core/resourceDescriptors.ts`,
  `src/routes/cloud/[resource]/`, `src/lib/domains/tasks/components/TaskDetail.svelte`,
  `src/lib/domains/scripts/components/ScriptForm.svelte`,
  `src/lib/domains/projects/pipelines/components/PipelineRunDetail.svelte`.

## Decisions already made, and why

- `ResponsivePanel side="right"` is the right-rail primitive. `ShellSidebarLayout` is
  left-only **by design** — adding a right variant was tried in an earlier wave and reverted.
- Four files are deliberately **not** split despite their size: `ProjectAudit` 960,
  `CoderPanel` 852, `sdk/ai/ollama` 880, `cloud/workloads/pods/[pod]` 907.
- Cloud detail pages stay at `/cloud/workloads/<plural>/[singular]`; only the _list_ routes
  collapsed. The redirect is `workloads/[...rest]` and must never swallow a detail URL.
- The deployments Scale button was dropped by a fixer and I had it **restored** as an opt-in
  `rowActions?: Snippet<[ICloudResource]>` on `BaseResourceTable`, hover-revealed with
  `opacity-0 group-hover:opacity-100 focus-within:opacity-100` so it stays in the tab order.
  A keystroke-only action is not a discoverable action.
- UX-016's primary actions stay in the `ProjectForm` / `TaskForm` / `DocumentEditor` footers
  on three routes rather than the header. Hoisting them needs those shared components
  refactored, and a long form with submit at the end is defensible. **Not a miss, do not file.**

## What bit me

- **A commit landed that no agent of mine made**: `d8ce6e3`, 11:43, sweeping waves 1-3
  together. Every fixer was told to commit nothing and none reported doing so. It is complete
  and correct so there is nothing to undo, but **the owner was never asked**. Check with them
  before pushing.
- Two fixers reached for an inline `--sidebar-width-domain` override; the first made the rail
  _wider_ than the narrow-screen rule, which is backwards for a small-screen brief. Rejected.
  That is what UX-020 is about.
- One agent "diagnosed" QA-051 and QA-052 from their stale Problem sections without noticing
  both are already `status: verified`. **Read the frontmatter status before trusting any
  agent's reading of a board file.**
- A fixer shipped a single shared search box across two tabs, which would have recreated
  QA-082 (no-match empty state claiming the list is empty). Caught and sent back.
- `pnpm lint` is red on this very file and was red at HEAD before any of this — ignore it.

## How to run the app

`node scripts/dev-instance.mjs 2` gives vite :1421, CDP :9223 and its own database. Instance 1
(:1420/:9222) is the owner's — never start, stop or drive it. Playwright over CDP, not pixels;
`.claude/qa/scripts/i2.mjs` is the instance-2 helper. `role="button"` wrappers swallow nested
buttons (QA-086), so assert `tagName === 'BUTTON'` when clicking.

Owner's process rule: reviewer -> fixer -> tester as separate agents; none grades its own
work. Ask before any page merge, move, delete or new route — route map first. Ask before
committing.

---

# Handoff 3 — Projects dashboard organization: SDLC stage + archive flag (appended 2026-11-10)

Separate workstream from everything above; those are finished (coder engine) or in their
own cadence (UX board). This one is the owner's ask to get their scattered project list
organized by state, so they can tell what's demo-ready, what to stabilize, what to park.

## Done this session

- Added a manual `stage` field: `idea | in_progress | stable | released`. Shown as a badge
  on `ProjectRow`/`ProjectCard`, set via each row's ⋯ menu → a "Stage" submenu
  (`DropdownMenuRadioGroup`), with a **read-only** hint line above the picker
  (`activity_status` / `risk_level` / unpushed / uncommitted) — evidence only, never
  auto-sets anything.
- Added an independent `archived` boolean — "don't consider this for next work until I
  un-flag it." Same ⋯ menu (Archive/Unarchive). Hidden from the list by default; a
  "Show archived" switch on `/projects` reveals it; a new "Archived" `PageStats` tile.
- **Correction made mid-build, by a reviewer agent, not by me up front:** `archived` does
  **not** reuse the existing `status` field (`active|archived|deleted`). The disk-scan
  import (`discovery_repository.rs::import`, `mark_missing`) already writes
  `status = "archived"` to mean "missing on disk this scan" and never clears it when the
  project reappears — reusing it for the manual flag would have conflated "I parked this"
  with "the scanner lost track of it," silently and invisibly, the first time the owner
  reorganized a folder the scanner had already walked. `archived` is its own column,
  written only by `set_project_archived`.
- Deliberately not built: a separate "priority" flag (`starred` already exists for that),
  group-by-stage (the filter already answers "show me Stable"), stage-change history, bulk
  stage/archive from the main list (only sketched as an optional step in the plan, from
  `/projects/audit`'s existing marked-selection panel — not implemented).
- Full plan, with every file:line target, at `C:\Users\tanvo\.claude\plans\quirky-sparking-castle.md`
  (local Claude Code plans dir, not in this repo).
- **Verified for real**, not just gated: drove a second dev instance (`:1421`/`:9223` —
  instance 1, `:1420`/`:9222`, is the owner's live app and was never touched) and called the
  Tauri commands directly through the page's own JS console (`window.__TAURI_INTERNALS__.invoke`).
  Set stage, cleared it, archived, unarchived, confirmed the row hides and reappears with
  both badges on the "Show archived" toggle, confirmed an unrecognized stage value is
  rejected with a clear error rather than silently written.
- Committed: `d81102a7` "feat(projects): manual SDLC stage and an independent archive
  flag" on `main`, 17 files. **Not pushed** — owner hasn't asked.

## Files that matter

- `src-tauri/portal-core/src/migrations/m20261110_000045_add_project_stage.rs`,
  `m20261110_000046_add_project_archived_flag.rs` — the two new columns, each independently
  `has_column`-guarded, each with its own doc comment explaining why it's a separate column.
- `src-tauri/portal-core/src/domains/projects/services/project_service.rs` —
  `set_project_stage`/`validate_stage`, `set_project_archived`. Same tier as
  `toggle_project_star`/`set_project_parent` — **not** the `discovery_*` triage machinery,
  which is a different concern (disk-scan audit workflow).
- `src-tauri/portal-core/src/domains/projects/repositories/project_repository.rs` —
  `set_stage`, `set_archived`.
- `src/routes/projects/+page.svelte` — `filterStage`, `showArchived` (default `false`),
  `hasHiddenArchived` (feeds a clearer `PageEmpty` message when a non-empty list is just
  entirely hidden behind the toggle, instead of claiming there are no projects at all).
- `src/lib/domains/projects/components/ProjectQuickActions.svelte` — the Stage submenu,
  hint line, Archive/Unarchive item.

## What bit me — read before you redo any of this

- **The status-field reuse mistake above.** Before reusing any existing enum-ish field for
  a new meaning, grep every other writer of that field first — don't assume "nothing else
  touches this" without checking. It cost a full correction cycle here.
- **`pnpm gen:types` had a silent regression.** Running it a second time in the same
  session (after the Rust side had already compiled correctly once) made the prettier
  post-pass in `scripts/gen-types.mjs` quietly stop reformatting ts-rs's raw output —
  no error, just 110 changed files across all four generated domains (coder/disk/github/
  projects), each collapsed to one line with double quotes instead of the committed
  multi-line/single-quote style. The script's own per-file timing output looked identical
  to a successful run either way. **Always check `git status --porcelain` after a type
  regen, don't trust the command's own output.** Fix used here: `git restore` the untouched
  dirs, hand-edit the one real field addition into the one file that actually changed.
- **The row-is-an-`<a>` trap.** `ProjectRow`'s whole row is a link, so nothing that needs
  its own click target (a `Select`, in this case) can nest inside it — same shape as the
  old QA-086 `role="button"` issue. That's why the Stage picker lives inside
  `ProjectQuickActions`'s dropdown instead of inline on the row.

## Next: safe project relocation (owner's spec — not yet planned or built)

The owner wants to **move a project's directory from inside Portal** as part of
organizing — consolidating scattered projects under fewer roots, moving things to line up
with a new stage. Requirements as given, intent preserved rather than paraphrased thin:

1. **Before moving:** check nothing is actively using the project — a running dev server
   for it, an open Coder thread/session against it, an open terminal cwd'd into it, anything
   else Portal itself currently has open on that path. The move must refuse to start if
   something is using it.
2. **During the move:** lock the project so **nothing else in Portal can open or use it**
   until the move finishes or aborts — not just a start-time check, active prevention of
   new use for the whole duration.
3. **The move itself** must be safe against partial failure: copy first, verify the copy,
   only then remove the original. Never move-in-place in a way that could leave neither a
   complete source nor a complete destination if interrupted — crash, disk full,
   permissions, an antivirus lock, the app itself closing mid-copy.
4. **Fallback:** something going wrong at any step needs a way back to a known-good state.
   At minimum: never delete the original before the destination is verified complete, and
   have a defined recovery path for an interrupted move — resume it, or roll it back, but
   never silently leave the project in a state Portal itself can't find or open again.
5. The owner said, and repeated: **"be very very cautious not to lose anything."** Treat
   that as the constraint every design choice below it has to satisfy, not a footnote to
   satisfy once and move past.

**Nothing here has been scoped yet** — no plan, no code. Before building, this needs its
own plan-mode pass, likely with `crew:architect` rather than `crew:planner`: it spans the
discovery/scan domain, the coder/terminal domains (for "is anything using this"), and raw
filesystem operations — not a single-subsystem change like the stage/archived work above.
Open questions a plan would need to settle, not yet decided by anyone:

- **Exact "in use" signal set.** At minimum: `dev_port`/`prod_port` actually listening (via
  what check — the SDK manager already tracks some service ports, worth reusing rather than
  reinventing), an open Coder thread with a matching `project_id`, an open terminal tab
  cwd'd under the path (**prefix** match — a subdirectory counts, not just an exact path
  match). Audit every place Portal can be "using" a project before finalizing this list;
  the stage/archived work above only touched the list page, this touches far more of the
  app's surface.
- **What "locked" means mechanically.** A DB flag (e.g. `projects.locked_at`) is the obvious
  shape, but it's only as good as every entry point that opens a project checking it —
  Coder, terminal, dev-server start, the project detail page's own actions, anywhere else
  that invokes against a project path. That's an audit of call sites, not a guess.
- **Verification after copy, before delete.** "Copy succeeded" from the OS isn't enough
  given the owner's framing — size/file-count comparison at minimum, a hash check for
  anything small enough to be worth it. Large `node_modules`/build directories are probably
  excluded from the moved set entirely rather than copied-then-trusted — the audit's own
  `regenerable_dirs` detection (`discovery/analysis.rs`, already used in `ProjectAudit.svelte`)
  already identifies exactly these; reuse it. Don't move what can be reinstalled, it's
  faster and categorically safer than moving it and hoping.
- **The interrupted-move recovery story**, decided explicitly before any code: e.g. a move
  writes a small marker (sidecar file at both ends, or a DB row) recording
  "move in progress: source X, dest Y, started at T" that a startup check can detect and
  either resume or flag for the owner to resolve by hand. Never have the app silently guess
  which side is authoritative after a crash mid-move.
- Whether this reuses any of the existing disk-scan/discovery machinery
  (`discovery_repository.rs` already walks paths and already tracks `discovery_path`
  separately from `path`) or needs to be wholly new, given it's a very different operation
  (scan-and-report vs. mutate-the-filesystem-and-never-lose-data).

This has not been discussed with the owner beyond their original one-message spec above —
a plan-mode pass should surface the open questions back to them before any code gets
written, given the stakes they themselves named.

## Starting fresh

Open a session in `D:\projects\production\portal\portal_desktop` and say:

> read .claude/handoff.md

Then decide: keep organizing with stage/archived (usable now, from each project's ⋯ menu),
or scope the safe-move feature above first — it needs a plan before it needs code.
