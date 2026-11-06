# Handoff — Portal Desktop coder engine work

Written 2026-11-06. For someone with no memory of the session that produced it.

## Goal

Two things the owner asked for, in order:

1. Merge the outstanding branches to `main` in both repos (portal_desktop and agent-platform).
2. Test Portal Desktop's **Coder** feature by driving the real app as a human developer would — pick a dummy project, brief it to build a React Native todo app, and find out what it can and cannot do. Fix what that test exposes.

A third item was added mid-session and is **not started**: test the coder on **Ollama via agent-platform**, to prove the coder is provider-independent rather than a thin wrapper around Claude Code.

## State of the repos

**portal_desktop** — on branch `feat/coder-cli-permission-mode`, cut from local `main`. Two commits on it:

- `dd2d844` fix(coder): let the Claude CLI engine run commands
- `fb4be1e` feat(coder): record file changes on the Claude CLI engine

**Nothing is pushed.** Local `main` (`6240016`) carries three merges: `feat/coder-claude-cli-engine`, `fix/pipeline-step-timeout`, `fix/ci-svelte-kit-sync`.

Three branches were deliberately NOT merged — do not merge them without re-checking:

| branch                     | why skipped                                                                                                                                                                        |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fix/p0-shell-exec-csp`    | Direct diff vs `origin/main` is _only_ a 2-line `package.json` change; the rest already landed as squash #42. Fully redundant.                                                     |
| `feat/local-docker-deploy` | Oct 17 base. Merging reverts 304 files (SvelteKit 3, shadcn re-vendor, P0 security work), downgrades version 0.12.3 → 0.12.2, and restores committed `target/rust-analyzer/` junk. |
| `feat/coder-agent`         | July base, 85 commits behind, pre-`portal-core` layout. The coder domain has been rebuilt since with +9160 lines.                                                                  |

Use **two-dot** diffs (`git diff origin/main <branch>`) not three-dot when judging redundancy — three-dot hides that a squash merge already landed the content, which is what made these look live.

**agent-platform** (`D:\projects\production\ai\agentic-ai\agent-platform`) — local `main` has `feat/coder-approve-json-twin` merged (`221f158`). Conflicts in `desktop/crates/app/src/main.rs`, `screen.rs` and `plan.md` were resolved **to the branch**: main still held the pre-reframe nav and dashboard that the branch deliberately replaced. Not pushed.

## SHARED CHECKOUT — read this before touching anything

Another session, "Tasks UI revamp and organization", is working in **this same checkout with no worktree**, by the owner's instruction. It has uncommitted work and has agreed to commit onto this branch.

Paths that belong to **them** — do not edit, and never stage:

- `src/lib/domains/tasks/**`
- `.claude/qa/**`
- `src/routes/github/issues/+page.svelte` (legitimate knock-on: that page reuses KanbanBoard/TaskList, so extracting shared helpers into `utils/taskDisplay.ts` forced a matching edit)

Paths that belong to **this** work, which they stay off:

- `src/lib/domains/coder/**`
- `src-tauri/portal-core/src/domains/coder/**`

**Always `git add <path>`, never `git add -A`.** One sweep commits the other's half-finished work. Both commits above were staged by name and verified afterwards with `git show --name-only`.

Contact them with `SendMessage` to `Tasks UI revamp and organization`. They have been reliable and proactive — they caught that an HMR reload would wipe a coder thread mid-test, which the session that wrote this had missed.

## What was found and fixed

### The test that exposed it

Scratch repo at `D:\projects\practice\todo-rn` (created for this, imported into Portal as a project). The coder was briefed like a human would brief a developer and **wrote a correct Expo/TypeScript todo app** — including, unprompted, a `loaded` gate in `App.tsx` so the initial empty state does not overwrite AsyncStorage on launch. Verified independently: 716 packages install, `npx tsc --noEmit` clean.

Then it stopped. It could not run a single command.

### Fix 1 — `dd2d844`, commands

Portal spawned `claude -p` with a hardcoded `--permission-mode acceptEdits` and no `--allowedTools`. Headless Claude cannot display an approval prompt, so it **auto-denies every tool that would otherwise ask**, Bash included. Worse, the permission-mode selector was hidden for this engine (`hideModes={isCli}` wrapped it), so no user could grant it.

Now Portal's existing permission mode maps onto the CLI flag — `Review` → `acceptEdits` (unchanged default), `AutoAcceptAll` → `bypassPermissions`, which is already what that mode means for the platform engine. The selector renders for both engines; only the _agent_ mode stays platform-only behind `hideModes`.

**This one is verified end to end.** Thread `cfa40a12` in the sqlite DB contains three real `Bash` tool calls including `npx tsc --noEmit`. The owner chose this approach over an `--allowedTools` allowlist.

### Fix 2 — `fb4be1e`, the Changes panel

The owner noticed the **Agent changes** panel stays empty for Claude Code sessions. Confirmed: `record_change` had exactly one call site, on the agent-platform path. Claude runs its own tool loop, so Portal never saw the write. (Git changes still worked — it shells out to git independently.)

Now reconstructed from the stream-json transcript: Claude emits `tool_use` _before_ running a tool and `tool_result` _after_, so the file is snapshotted on the first and recorded on the second, by which point `record_change` can read `after` off disk as it already does.

Three pure helpers next to `parse_cli_event` in `service.rs`, unit-tested the same way: `cli_edit_targets`, `cli_tool_result_ids`, `workspace_relative`.

**Known limits, stated in the doc comment:** the snapshot-to-write gap is a race Portal cannot close while Claude owns the write; files edited outside the workspace root are skipped.

## What is left

1. **Confirm the Changes panel actually populates.** This is the only unverified part of `fb4be1e`. The Rust is live in the running app, but nobody has watched the panel fill. Do it by starting a coder session on `D:\projects\practice\todo-rn`, asking for a small edit, then checking the panel — and verify at the data layer too:

   ```
   sqlite3 <db> "select count(*) from coder_file_changes where thread_id = (select id from coder_threads order by rowid desc limit 1);"
   ```

   It was **0** before the fix. DB path: `C:\Users\tanvo\AppData\Roaming\com.tanvoid0.portal-desktop\portal_desktop.db` (copy it before querying; the app holds it open).

2. **Ollama via agent-platform — not started.** `agent-platformd` is healthy on `http://127.0.0.1:18410` but requires `Authorization: Bearer <session JWT | agp_ workspace token | master key>`. Ollama is **not running**; binary at `C:\Users\tanvo\AppData\Local\Programs\Ollama\ollama`, and the only model on disk is `gemma4`, which may not support tool calling — the platform engine needs function calling, so pull something like `qwen2.5-coder` if it does not. Then fill the **AgentPlatform** provider row in Settings > AI (`base_url`, `api_key`, `model`, and it must be _enabled_) and start a **new thread**, because the engine is stored per-thread in `coder_threads.llm_provider`.

3. **Push.** Nothing is pushed in either repo. The owner has not asked yet.

4. **Loose end, not investigated:** the coder's project dropdown showed a stale Linux path (`/media/tan/Workspace/...`) for the Portal Desktop project itself, on a Windows machine.

## Things that bit the last session

- **Counting `cargo`/`rustc` processes is not a build check.** `tauri dev` leaves a `cargo run` alive as the parent of the app, with near-zero CPU. A whole stretch of the session was spent reporting a "stuck 25-minute rebuild" that had finished in 46 seconds. Check **CPU time** and the **binary mtime vs source mtime** instead. Better still, grep the built binary for a string only the new code contains.
- **Running `cargo check --workspace` concurrently with `cargo test` against the same target dir produced a phantom `16 passed / 1 failed`.** Two clean runs after showed 215/0. Do not chase a lone failure under concurrent cargo.
- **`pnpm check` and `pnpm lint` cover the whole tree, including the other session's in-flight work.** A red result is ambiguous between the two sessions. For a Rust-only change, run the cargo gates and say why you skipped the frontend ones.
- **The app is a Tauri window, not a page** — drive it as a window. There is a helper at `C:/Users/tanvo/AppData/Local/Temp/claude/6ce5694c4c5839e1/5c08034c-1f05-47a7-a84a-86a304b95e74/scratchpad/pd.sh` (`shot` / `click` / `type` / `key` / `scroll` / `title`), but it lives in a session-scoped temp dir and **may be gone** — if so, see `C:/Users/tanvo/.claude/plugins/cache/tanvoid0/hands/0.5.1/win.ps1` and its `stacks/tauri.md`, which `pd.sh` wraps. Window coordinates include the title bar. `title` reads the document title and is the cheapest navigation assertion available. Native file dialogs are separate windows: set the path with `WM_SETTEXT` on the filename Edit, then `BM_CLICK` the confirm button.
- **Screenshots are expensive and unevictable.** Delegate "look at this and tell me what it says" to a subagent so the image lands in its context, not yours.
- **Do not trust the agent's self-report.** It claimed an exit code; the claim was only confirmed by finding the actual `Bash` tool call in `coder_threads.messages_json`. The same lesson applies to the panel: "the panel is empty" and "the write never happened" are different failures.
