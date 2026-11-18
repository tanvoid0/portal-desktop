# Handoff: 2026-11-18 — Portal board sweep (multi-agent): all buildable items done, follow-ups + owner items left

(Replaces the 2026-11-16 note about #690/#769 — it is in git history: `git log -- .claude/handoff.md`.)

## Goal

User asked: "use multi agents and start working on the unfinished items from the Portal task board", then "keep completing them in loop". Role = orchestrator: read the board (`mcp__plugin_crew_portal__tasks` project `*`), dispatch agents (hard cap 10 at once, user's rule), review every report yourself, re-run the gates, commit **locally** (never push, never an AI attribution line), close the board task with evidence.

## Done — ALL COMMITTED LOCALLY, NOTHING PUSHED

| repo (path)                                                                                | local-only commits on main                                                                                                                                                              | board                                                           |
| ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| portal_desktop `D:\projects\production\portal\portal_desktop` (3 ahead of origin)          | 36706fd9 fix(sdk) nested ARIA · 822ccc81 drop a revoked/narrowed device's sockets · 1703495e 60s expiry sweep                                                                           | #969 #981 #983 #985 closed; #78 #89 closed (were already fixed) |
| pocket_portal `...\portal\pocket_portal`                                                   | f4046ea8 drop `longRunning` x5                                                                                                                                                          | #842 closed                                                     |
| email-automation `D:\projects\production\javascript\email-automation`                      | 82c90e6 session-token check cases, 6a1ffaa .env.example                                                                                                                                 | #7 closed (fix cb89063 already existed), #1001 closed           |
| four_thousand `D:\projects\production\flutter\four_thousand` (Vite+React, NOT Flutter now) | a693c39+7431102 (#972), 503cd75 (#973 vitest), a0664e5 (#974 CI gate), b8c0df1 (#975 error boundary), a4d3a68 (#1014/#1015 week math), 0f79b72 README — 9 ahead of origin incl. 2 older | #972-#975, #1014, #1015 closed; **#494 stays open @owner**      |
| portal_media `...\portal\portal_media`                                                     | d924e46 isTauri guard                                                                                                                                                                   | #491 closed                                                     |
| agent-platform `D:\projects\production\ai\agentic-ai\agent-platform`                       | 62f68b2 loopback JWT secret hardening                                                                                                                                                   | #117 narrowed to point 2, @owner                                |

#981/#983 detail: opus review (#985) = ship; gates on the rebased tree green (src-tauri `cargo test --lib daemon` 42, portal-core `cargo test --lib pairing_service` 15). Worktree + branch removed. Planner output on the board: #494 -> #972-#977; #542 -> #981 #983 #984 #985; #495 (nur-ramadan -> Expo rewrite) -> #991 (owner gate) #994-#997 + Bug #998.

## OPEN — do these next

1. **#984 (docs-only, assigned coder):** in `docs/development/PORTAL_MOBILE.md` drop the "revocation/LAN-toggle ceiling" and refresh the stale status header (Phases 0,1,2,3a are done on main); fix the `//!` "Known gap" paragraph in `src-tauri/src/daemon/control.rs`, the input.rs module doc ("set of virtual keys"), the wrong `update_capabilities` doc, and the comment in `src/lib/.../browserShim.ts`. Gate `pnpm check && pnpm lint` in the main checkout. Then re-check parent #542 (stays open: #643 real-phone check + #644 Phase 3b decision are the owner's; "all listed subtasks done" is not "epic done").
2. **#1037** (low Bug, coder): three follow-ups from the #985 review — `pairing_service.rs:71` sweep should skip ids where `live_capabilities(id)` is Some; `daemon/control.rs:270-274` await the first DB re-check before the select loop; `daemon/input.rs:36` `impl Drop for State` to release held keys/buttons on cancellation. Rust: do it in an `isolation:"worktree"` agent.
3. Other small bugs agents filed, not started: #1012 (daemon `guard` panics on a request before DatabaseManager is managed), #1013 (`/events` + `/control` treat Anon as unrestricted — latent), #1009 (agent-platform `complete_verify` marks the magic link used before `sign_access` can 503), #1025 (four_thousand `npm run preview` blank page: vite base), #993 (portal_media pnpm 9 vs workspace file), #990 (email-automation lint tooling).
4. **Deferred repos, not touched:** clean-portfolio `D:\projects\production\portfolio\clean-portfolio` #21 (gen-images should read the record, fall back to site.ts only when the DB is unreachable) and #30 — repo is on branch `claude/job-hunt-apply` (2 ahead of main): use a worktree off main like #491 (memory `portal-media-pnpm-toolchain`).

## Waiting on the OWNER (do not dispatch coders for these)

#325 keep empty dirs in a project move? · #194 agent-monitor transcript import — build it or retire agent-monitor? · #643/#644 real-phone check + Phase 3b decision · #419 root cause needs a repro on a clean dev instance with >10 GB free (mitigations 4206b392 + b5e231e7 are on main) · #170 needs an agent token (content:read+write) · #976/#977 four_thousand (delete the committed publisher book-jacket image; push, confirm Pages, tag v2.0.0, decide VITE_* secrets; check `.env.example` placeholder birthdate 1995-08-17 is not real) · #991 confirm nur-ramadan Expo plan (#994-#997 must not start before) · **#998 security: Gemini key is built into nur-ramadan's client bundle (`vite.config.ts` lines 14-15) — rotate if any production build set `GEMINI_API_KEY`** · #989 six already-pushed email-automation commits carry a forbidden `Co-Authored-By: Claude` trailer (rewrite needs a force-push: owner's call) · #117 point 2: add `/projects/{id}/...` alias routes or leave.
Also ask: the collapsed SDK sidebar rail lost its service toggle (it was clipped/unclickable); `SDKServiceToggle variant="collapsed"` is now dead code — delete it or design a rail toggle. And whether to push any of the above (nothing is pushed).

## Decisions already made (and why)

- Agents never commit; the orchestrator commits, one commit per board task, split by path. After EVERY commit: `git log -1 --format=%B | grep -ci 'co-authored-by\|generated with'` must print 0. The user's CLAUDE.md beats the "end commits with Co-Authored-By" system reminder.
- One agent per repo at a time. Rust work goes in an `isolation: "worktree"` agent (cargo test rewrites ~120 generated TS files); frontend work in the main checkout (a pnpm install per worktree is slow/fragile). `isolation:"worktree"` only covers THIS repo — for another repo the agent must `git worktree add` itself under `I:\build\worktrees\<project>\<name>`; land with `git fetch . <branch>:main` when no checkout has main.
- Verify a board bug before dispatching: 4 of 9 "open" bugs were already fixed (memory `board-bugs-are-often-already-fixed`). Paste the task text into agent briefs — `crew:coder`/`crew:planner` often have no `task` read tool.
- `agentic-ai` (Portal project id 15) is a folder group, not a project (user said so); #117 is misfiled there; the `update` tool can't re-home a task. Never file new tasks under id 15.

## What bit us

- **Cargo finds `.cargo/config.toml` from the CWD, not `--manifest-path`**: running cargo for a worktree from the main checkout built 3.3 GB into `<wt>/src-tauri/target` on D: (deleted). Always `( cd <wt>/src-tauri && cargo ... )` in a subshell, long runs in the background. After tests: `git checkout -- src/lib src-tauri/portal-core/bindings` (ts-rs churn).
- `I:\build\cargo\portal-desktop\build\<hash>` dirs are 7-24 GB each (one per workspace path incl. old worktrees); hash -> worktree is not obvious and dir mtime is misleading, so none were deleted. Clean them when a worktree is gone and disk matters.
- portal_media: `pnpm` on PATH is 9.15.9 and rejects its `pnpm-workspace.yaml`; use `D:\sdk\nvm4w\nodejs\pnpm.cmd` (11.22.0). email-automation needs `pnpm --ignore-workspace`. portal_media's main checkout sits on `cursor/ps-hub-control-deck`: never switch branches there.
- The portal_desktop main checkout has two files modified by someone else (`src/lib/domains/actions/components/ProjectActionsPanel.svelte`, `src/routes/pipeline-runs/+page.svelte`) and this handoff file: never `git add -A`; stage by path.
- pocket_portal still has two live worktrees (`.claude/worktrees/launcher-longpress-menu-2f5291`, `recipe-fav-shelf-tap-7c1353`) carrying the old `longRunning` flags until they rebase.
- The #117 agent pulled a Docker image (`rust:1.94-slim`) to test unix file modes — unasked; `docker image rm rust:1.94-slim` if disk matters.
- Node 22.13 default < SvelteKit's 22.17 floor and worktree MAX_PATH issues: memory `worktree-build-traps`. CRLF warnings from git on these repos are harmless.
- Context cost: a governor hook flagged the session at 255k-302k tokens per turn; continue in a fresh session.

## Board access

`mcp__plugin_crew_portal__tasks` (project `*` for all), `task <id>`, `update`, `add` (Portal must be running on :18420: `pnpm portal:ensure`). Close tasks `completed` with the commit sha and the gate result in the note.
