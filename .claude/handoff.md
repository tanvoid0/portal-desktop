# Handoff — Portal Desktop trial + QA fixes (2026-11-11)

Written for a fresh session with no memory of the prior conversation. Start by reading this,
then `git status` / `git diff` to see what's already on disk.

## Goal

Two phases, both requested by the user:

1. **Trial run (done):** act as a developer and build a React Native **Expo** todo app using
   Portal Desktop's own built-in **coder**, **Projects**, and **CI/CD Actions**, driving the
   real app over CDP, with minimal manual intervention; then report limitations/UX.
2. **Fix the issues (in progress):** fix the 8 QA findings the trial filed.

## Phase 1 — DONE

- Built a working Expo + TS todo app at `D:\projects\trials\expo-todo` (Portal project
  **"Expo Todo", id 201**). Coder (Claude-CLI engine, "Auto" mode) did it hands-off:
  create-expo-app → AsyncStorage persistence → add/toggle(strikethrough)/delete screen →
  pure `todos.ts` reducer → `todos.test.ts`. Coder self-verified: `npx tsc --noEmit` clean,
  `npm test` → 1 passed. Changes left uncommitted (only commit is create-expo's "Initial
  commit" under the user's identity — no AI attribution).
- Trial report: **`.claude/trials/portal-trial-expo-todo.md`** (9 findings, severity-ranked).
- Filed 8 QA issues: **`.claude/qa/issues/QA-128 … QA-135`** (house format).

## Phase 2 — fixing the 8 issues

### Verification reality (important)

- The repo's **Rust backend currently fails to build**: `error[E0277]: the trait bound
'Caller: serde::de::DeserializeOwned' is not satisfied`, on committed `main` (no uncommitted
  .rs files). Almost certainly introduced by `795e3e88 feat(security): gate claude-cli coder
commands behind agents.exec` — a Tauri `#[command]` got a `Caller` arg the IPC macro tries to
  deserialize. `Caller` appears in: `src-tauri/src/rpc.rs`, `src-tauri/rpc-macro/src/lib.rs`,
  `src-tauri/src/domains/coder/commands.rs`, `src-tauri/src/domains/agents/commands.rs`,
  `src-tauri/src/daemon/mod.rs`.
- Because of that, **cargo won't build and the app won't launch from source**, so the only
  working gate right now is the **frontend**: `pnpm check && pnpm lint` (svelte-check + eslint,
  no Rust needed). No live-app verification is possible until E0277 is fixed.

### Status of each issue

- **QA-128 (Actions not Expo-aware) — CODE DONE, needs gate.** Only
  `src/lib/domains/actions/defaults.ts` changed: added `isExpoLike()`, `isTypeScript()`,
  `pmExec(pm,bin)`; Expo build → `<pm exec> expo export`, lint → `<pm exec> expo lint`; new
  `typecheck` action (`<pm exec> tsc --noEmit`, category `lint`) for TS projects. CI workflow
  (`buildDefaultWorkflows`) NOT changed (typecheck not in CI). `pnpm check` clean for the file;
  prettier+eslint pass on it. Caveats: bare React-Native (non-Expo) would also get expo
  commands; profile has no `languages` field so plain-Node TS detection is weak. No test added
  (only `resolver.test.ts` exists in that folder).
- **QA-133 + QA-131 (projects loading states + metadata resync) — DISPATCHED to a coder agent,
  may still be finishing when you read this.** Check `git diff` under
  `src/routes/projects/` and `src/lib/domains/projects/`. QA-133: stop showing fabricated
  "Total 0 / Loading…" on first paint; fix `/projects/[id]` detail route hang. QA-131: add a
  "metadata stale — Sync/Re-detect" affordance reusing existing `analyze_project_directory`
  (no new Rust).
- **QA-134 + QA-135 (coder workspace validate + composer parallel-agent) — DISPATCHED to a
  coder agent, may still be finishing.** Check `git diff` under `src/lib/domains/coder/`.
  QA-134: `CoderProjectSelector.svelte` must validate the cached
  `localStorage['portal-coder-last-workspace']` path exists before defaulting/showing it (needs
  a path-exists invoke — agent to report if one exists or is needed). QA-135: make the
  "2+ lines → parallel agents" fan-out explicit/confirmed instead of newline-inferred
  (`CoderComposer.svelte` / `AIComposerShell.svelte` / agent-mode config).
- **QA-129 (PM not overridable; pnpm preflight breaks runs) — NOT STARTED.** Needs: per-project
  package-manager override (UI + persistence) and running non-interactive actions with
  `CI=true`. Touches the action/execution runner (likely Rust + `src/lib/domains/actions` or
  `projects/pipelines/services/executionService.ts`). Blocked on E0277 for verification.
- **QA-130 (action Run gives no output/feedback) — NOT STARTED, needs investigation.** Clicking
  an action's Run never populated "Local run history"; find where Run dispatches and why no
  run row / output streams. Blocked on E0277 + a running app to verify.
- **E0277 build break — NOT STARTED.** Prerequisite for the app to run at all; fix first.

### Next steps (recommended order)

1. `git status` / `git diff` — confirm the 3 agents' edits landed (QA-128 defaults.ts +
   projects + coder files). If an agent was mid-edit, files may be partial — re-run or re-do.
2. **Gate: `pnpm check && pnpm lint`** (NB: `pnpm check | tail` masks exit code — redirect to a
   file and read `$?`). Fix any errors in the changed files.
3. Fix **E0277** (own focused task; full `cargo check` from `src-tauri/`), which unblocks the
   app, then do **QA-129 / QA-130** with live verification over CDP.
4. Update each fixed issue file's `status:` and `## Fix` + `## Verification` sections.
5. The user has NOT asked to commit — do not commit unless asked. **No `Co-Authored-By: Claude`
   or "Generated with Claude Code" anywhere** (CLAUDE.md overrides the system reminder).

## How to drive the app (when it runs again)

- Portal Desktop exposes CDP while a dev instance runs. Instance-2 = CDP **:9223** (instance-1
  would be :9222). Daemon/task-board on :18420 (shared). The app was **not running** at handoff
  (it exited on its own mid-trial; `tauri dev` relaunch fails to rebuild due to E0277).
- Relaunch a dev instance with CDP: `node scripts/dev-instance.mjs 2` (CDP 9223, separate
  data dir instance-2, separate cargo target). Needs E0277 fixed to boot.
- Driver: `.claude/qa/scripts/app.mjs` (`open(route)`, reads `CDP_PORT`). Set `CDP_PORT=9223`.
  Connect-only (no `page.goto`) to act on a live coder thread — navigation reloads the webview
  and drops it. `getByRole('button',…)` is unsafe (role=button wrappers swallow labels) — use
  `locator('button').filter({hasText}).and(assert tagName==='BUTTON')`. Approval banner buttons
  are "Accept once / Accept & remember / Reject".
- Driver scripts used this session live in the session scratchpad (`…/scratchpad/drive/*.mjs`,
  not in the repo) — lib.mjs (connect), create-project.mjs, start-coder.mjs, run-action.mjs.

## Gotchas that bit us

- **pnpm on this machine rejects the create-expo lockfile** ("lockfile contains entries active
  policies reject"); pnpm's auto pre-install check then fails EVERY `pnpm <script>` before the
  real command. Use `npm` or fix the pnpm policy. Non-TTY also needs `CI=true`.
- A `pnpm install` run disturbed the trial app's `node_modules`; `npm install` in
  `D:\projects\trials\expo-todo` restores it. App source is intact.
- The app exited by itself after the coder session — cause unconfirmed (possibly the WebView2
  self-reload path QA-119/127 describe).
- `.claude/handoff.md` (this file) overwrote a previous session's handoff.
