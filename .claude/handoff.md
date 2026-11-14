# Handoff: browser/other-desktop Portal Mobile client

Worktree: `D:\projects\production\portal\portal_desktop\.claude\worktrees\eager-mcclintock-79e007`
Branch: `claude/eager-mcclintock-79e007` (NOT pushed). Board task: #542 (project portal_desktop, in-progress, assignee claude).

## Goal

Extend Portal Mobile (phase 0 = Android thin client, already on `main`) so any
browser on the LAN — another desktop, laptop, tablet — can open the daemon's
URL and get the full Portal UI with zero install, paired by token. Full spec:
`docs/development/PORTAL_MOBILE.md`'s new "Other clients" section (written
this session, has the final design).

## Done (4 commits on this branch, in order)

1. `c8f868ec` — daemon static route (`src-tauri/src/daemon/mod.rs`): serves
   `build-client/` via `tower-http::ServeDir` as a `fallback_service` added
   _after_ `.with_state()` so it sits outside the `guard` middleware (public,
   like any static site — no secrets in it). Also closed a real gap the
   reviewer found: `classify()`'s loopback free-pass only checked `Origin`,
   but a same-origin `fetch()` from this new page can omit `Origin` entirely
   — added a `Sec-Fetch-Site` check alongside it (see module doc comment at
   top of the file, and the two tests `loopback_with_sec_fetch_site_does_not_
get_the_free_local_pass` / the pre-existing Origin one right above it).
   Also: `src-tauri/Cargo.toml` (+`tower-http`), `src-tauri/tauri.conf.json`
   (`bundle.resources` + `beforeBundleCommand: "pnpm build:client"`),
   `build-client/.gitkeep` (tracked — `tauri-build` validates
   `bundle.resources` paths on _every_ `cargo check`, not just packaging, so
   this must exist even before anyone's run the real build).
2. `41d8b2f9` — build tooling: `vite.config.ts`'s adapter now reads
   `VITE_BUILD_OUT_DIR`; `scripts/build-client.mjs` (new) builds with
   `VITE_PORTAL_MOBILE=1` + that var set to `build-client`, then restores
   `.gitkeep` which `adapter-static` deletes on every build; `package.json`
   gained `build:client`.
3. `421796b7` — `src/lib/domains/mobile/browserShim.ts` (NEW, ~290 lines):
   installs `window.__TAURI_INTERNALS__` (and `__TAURI_EVENT_PLUGIN_
INTERNALS__`) only when `isMobileShell` and nothing's there already, so
   the real Android Tauri runtime is never touched. Mirrors `mobile/src-
tauri/src/lib.rs` exactly: `mobile_status/_pair/_connect/_disconnect/
_discover` run locally against `localStorage` (not a file); everything
   else → `POST /rpc/{cmd}` with bearer token after the same command-name
   check (`src/lib/domains/mobile/commandName.ts`'s `isForwardableCommand`,
   ascii alnum/underscore only — rejects `plugin:barcode-scanner|scan` etc.
   with a clear error); `/events` → plain `WebSocket` with `?token=`, capped
   backoff reconnect, restores a saved connection on load. Wired into
   `src/routes/+layout.svelte` as the very first import. `src/routes/connect/
+page.svelte` prefills host/port from `location.*` when running as this
   shim (daemon has no CORS layer, so any other origin's `fetch` just fails).
4. `65bb5acd` — docs.

**Reviewer pass** (crew:reviewer, this session) found 7 issues against the
pre-fix shim; all 7 were fixed before the commits above landed (so the
commits already reflect the fixed version — nothing further to apply from
that review). Highlights if you want to re-verify independently: (a) the
shim didn't restore a saved connection on page load — fixed by calling
`bridge.set(saved)` in `installBrowserShim` when `loadConn()` finds one; (b)
a stale async chain from a superseded `set()`/`clear()` could still write
`caps`/`live` — fixed with `generation` checks right after the `whoami`
await, both success and catch paths; (c) the loopback same-origin trust gap
above; (d)-(g) smaller: missing `__TAURI_EVENT_PLUGIN_INTERNALS__`,
`adapter-static` deleting `.gitkeep`, a `vite.config.ts` comment nit, and an
uncaught synchronous `WebSocket` constructor throw.

`cargo check` and `cargo test daemon::` both pass (34 daemon tests, up from
33 — added the `Sec-Fetch-Site` regression test). Frontend is **not**
verified in a real browser — see blockers below.

## Left to do

1. **In flight right now, likely done by the time you read this**:
   `src-tauri/.cargo/config.toml` sets `target-dir = "I:/build/cargo/
portal-desktop"` (final path — went through two earlier revisions this
   session: first `C:/cargo-target/portal-desktop`, then `I:/cargo-target/
portal-desktop`, then grouped under the new global convention below). A
   `cargo check` + `git commit` for it was launched in the background right
   before this handoff was last edited; **confirm the commit actually
   landed** (`git log --oneline -1` should show "build(cargo): share one
   target dir across worktrees..."). If it didn't, the config file edit is
   still there on disk either way — just check/commit it.
   Also added a global convention to `C:\Users\tanvo\.claude\CLAUDE.md`
   ("# Build output" section, new this session): default any build/cache
   dir, across any project, to `I:\build\<tool>\<project>` — I: has ~932GB
   free, by far the most on this machine. Apply that convention to other
   projects' build dirs too if asked, not just this repo.
2. **This worktree's own stale `src-tauri/target/` (9.7GB, on the nearly-full
   D: drive) is now orphaned** — nothing points at it once the shared I:
   dir is in place. Safe to delete (pure build cache, rebuildable) to
   reclaim D: space; not deleted yet. Also disposable, also not deleted:
   `C:/tmp/cargo-target-eager-mcclintock`, `C:/cargo-target/portal-desktop`,
   `I:/cargo-target/portal-desktop` — all superseded intermediate attempts
   from this session's back-and-forth on where the target dir should live;
   only `I:/build/cargo/portal-desktop` is current.
3. **node/pnpm is broken machine-wide** on this box — filed as board bug
   #552. `pnpm install`, `npx`, and `corepack` all fail with `MODULE_NOT_
FOUND` under `D:\sdk\nvm4w\nodejs` and the pnpm global store (looks like
   an interrupted version switch, possibly by another concurrent session —
   do not try to repair it unilaterally; it's shared infrastructure). This
   blocks `pnpm check`, `pnpm lint`, `vitest`, and `pnpm tauri:dev` — i.e.
   **no part of the frontend change has been run or browser-verified**,
   only read carefully and reviewed. That is the single biggest gap before
   this can be called done. Whoever picks this up should check bug #552
   first; if node/pnpm is fixed, the real remaining work is: `pnpm install`,
   `pnpm check && pnpm lint`, `pnpm build:client`, then `pnpm tauri:dev` and
   actually pair + exercise a few `/rpc` calls from a browser tab per the
   original task's verification bullet.
4. D: was also separately logged full (643MB free of 450GB) as board bug
   #555 — unrelated to pnpm, just disk space. The I:-drive Cargo fix above
   addresses _this worktree's_ contribution to that; it does not fix pnpm,
   and other worktrees/sessions' own disk usage was deliberately left alone
   (not mine to clean up unilaterally on a shared machine).
5. Branch is **not pushed**, no PR, not merged. Tried: `git push` fails —
   this repo's husky `pre-push` hook runs `pnpm check && pnpm lint`, which
   fails immediately on the broken pnpm (same root cause as bug #552). Asked
   the user: skip the hook once (`--no-verify`) vs wait for pnpm to be
   fixed. **They chose: wait.** Do not `--no-verify` this push without
   asking again. Once node/pnpm (#552) is actually fixed: `pnpm install`,
   `pnpm check && pnpm lint`, `pnpm build:client`, exercise it in a browser
   per item 3 below, _then_ `git push -u origin claude/eager-mcclintock-
79e007` (should go through the hook cleanly then), open a PR, merge to
   `main` (confirmed a clean fast-forward as of this writing — `origin/main`
   is at `015e2663`, this branch is that +6 commits, no divergence to
   rebase). That whole sequence is what "merged to main" still needs.

## Key decisions already made (so you don't re-litigate them)

- Output dir is `build-client/`, not `build-mobile-web/` — user explicitly
  asked for this rename mid-session ("why not name it build-client? ... web,
  desktop, mobile everything that a client needs"). If you see
  `build-mobile-web` anywhere, it's stale/missed — grep for it.
- Static route is unauthenticated by design (outside `guard`) — the shell
  has no secrets; auth happens at `/pair/request` and every `/rpc` call.
- `build-client/.gitkeep` must stay tracked — don't gitignore it away, don't
  delete it, and `build-client.mjs` must keep recreating it after each real
  build.
- Shared Cargo target dir is a per-machine, uncommitted-concern path (it IS
  committed, since `.cargo/config.toml` needs to be in every worktree's
  checkout — but the _path itself_ is this-machine-specific, documented in
  the file's own comment; override via `CARGO_TARGET_DIR` env var if this
  ever needs to run somewhere else).

## Gotchas

- `cargo check`/`cargo test` run fine from `src-tauri/`; don't `cd` to the
  repo root first.
- A plain `grep -rn` across `src-tauri/src` with no bound gets blocked by
  this environment's hook — use the `Grep` tool or bound `head_limit`.
- Board tool (`mcp__plugin_crew_portal__update`/`add`) rejects any `note`/
  `title` over ~200 chars with a misleading "title too long" error even when
  you only passed a note — keep both short.
- `git mv` fails with "not under version control" for files that were never
  `git add`ed yet (e.g. renaming `build-mobile-web/.gitkeep` to
  `build-client/.gitkeep` before the first commit) — just `mv` directly.
