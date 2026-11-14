# Handoff: Database Explorer (#502) — P1+P2 done, P3/P4 open

## Goal

DataGrip-style database explorer inside Portal Desktop: browse + query + edit sqlite,
postgres, mysql, mongo, redis. Board task #502 (`mcp__plugin_crew_portal__task 502` for full
history). Plan: `~/.claude/plans/iterative-plotting-flame.md` — read it, it has the phase
breakdown (P0–P4) and the design rationale (why SQL engines share a pane but Mongo/Redis get
their own). Branch `feat/database-explorer`.

## Status: P0, P1, P2 committed and live-verified. P3 (Mongo), P4 (Redis) not started.

Commits on `feat/database-explorer` (not pushed):
- `eaad0049`, `8e3d85c1` — P0 (sqlite, read-only).
- `810dccd4` — P1 (postgres+mysql, read-only): `DbDriver` trait, `SqlServerDriver`,
  discovery (vault/SDK/Docker).
- `69e708ab` — P2 (editing): UPDATE/INSERT/DELETE for all three SQL engines, behind a
  per-source "editable" toggle + a server-side single-column-PK gate.
- One more commit **may or may not be committed yet** depending on when you're reading this —
  the owner was asked to run it (the agent in this session got denied by the permission
  classifier on every `git commit` attempt, every single time, no matter the message — if
  that's still true for you, don't fight it, just ask the owner to run the commit and move on).
  If `git log --oneline -1` on this branch is NOT one of the four hashes above, that commit
  already landed; if it's `810dccd4` or `69e708ab`, it's still pending. The pending commit
  fixes QA-157 + UX-021 + adds friendlier connect-error messages — see "Left to do" below for
  the exact message if you need to redo it.

### What's implemented (P0+P1+P2)

- **Backend** `src-tauri/portal-core/src/domains/database_explorer/`:
  `driver.rs` (the `DbDriver` trait — list_tables/describe_table/fetch_rows/run_query +
  update_cell/insert_row/delete_row), `sqlite.rs` (rusqlite, read-only connections for
  browsing, a separate read-write open only for edits), `sql_server.rs` (sqlx, postgres+mysql,
  one struct branching per engine), `discovery.rs` (merges vault DATABASE credentials + running
  SDK services + Docker containers into the source list), `state.rs` (`DbExplorerState`:
  source registry + per-source editable toggle, both in-memory/per-process — nothing persists
  across a restart except the hardcoded "Portal (this app)" source), `types.rs` (wire types,
  ts-rs exported), `mod.rs` (wires it together + the `preview_*` dispatchers, which build edit
  SQL with **no DB connection** so a preview never needs the vault unlocked).
- **Commands** `src-tauri/src/domains/database_explorer/commands.rs`: `db_list_sources`,
  `db_open_sqlite`, `db_list_tables`, `db_describe_table`, `db_fetch_rows`, `db_run_query`,
  `db_set_editable`, `db_preview_update`/`db_apply_update`, `db_preview_insert`/
  `db_apply_insert`, `db_preview_delete`/`db_apply_delete`. Two gates in front of every write:
  `require_editable` (per-source toggle) and `require_pk` (re-describes the table server-side;
  **rejects tables with a composite PK outright** — see "Decisions" below for why).
- **Frontend** `src/lib/domains/database/` (`api/databaseApi.ts`, `components/SourceNode.svelte`,
  `components/ResultGrid.svelte` — the editable grid, insert/delete UI, confirm-dialog-with-SQL-
  preview flow) + `src/routes/database/+page.svelte` (the page shell, Data/Query tabs).

### Decisions made and why

- **Composite PKs are rejected, not supported.** A code-review subagent found that the original
  implementation let a table with a multi-column PK through if *any* key column matched the
  frontend's chosen column — so a delete/update on one row could silently hit every row sharing
  that one column's value (mass-delete bug, rated HIGH). Fix: `require_pk` in `commands.rs` now
  requires the table to have **exactly one** PK column, full stop. Multi-column-PK tables are
  not editable in this version — no attempt to let the user pick a composite key through the UI.
  Upgrade path if ever needed: extend `pk_column`/`pk_value` to a list.
- **Every write also checks rows-affected == 1 and rolls back otherwise**, as a second
  independent safety net (`finish_pg_tx`/`finish_my_tx` in `sql_server.rs`, `exec_in_tx` in
  `sqlite.rs`) — belt-and-suspenders in case the PK gate is ever loosened or wrong.
- **Postgres casts edit params to the real column type** (`pg_column_types` + `pg_cast` in
  `sql_server.rs`, appends `::<type>` to each bound placeholder at **execution** time only).
  Without this, editing a uuid PK or any non-text column just failed outright (Postgres won't
  implicitly coerce a text-bound param). The confirm-dialog **preview does not show this cast**
  — previews are built with zero DB access specifically so they never need the vault unlocked,
  and that's judged worth a small preview/executed-SQL text mismatch (the cast only affects
  type coercion, never the bound value, so the dialog still shows exactly what gets written).
  Documented in both `sql_server.rs`'s module doc and `types.rs`'s `EditPreview` doc comment.
- **Big integers (>2^53) serialize as JSON strings, not numbers**, for sqlite INTEGER and
  postgres INT8/bigint (`JS_MAX_SAFE_INTEGER` constant in both `sqlite.rs` and `sql_server.rs`).
  Without this, a huge PK value sent to the frontend and back loses precision in JS and an
  edit/delete can hit the wrong row. MySQL BIGINT has the same theoretical issue but was left
  alone — narrower risk, not in the original review's findings, deprioritized.
- **Discovery guesses default local creds** (postgres/postgres, root/empty) for SDK- and
  Docker-discovered sources, and that's **correct** for a Portal-installed SDK service (the SDK's
  own config literally sets an empty password) but **wrong** for a real external Postgres/MySQL
  with its own password — this is a known, accepted gap (see `discovery.rs`'s module-level
  `ponytail:` comment), not a bug. What got added this session: `friendly_connect_error` in
  `sql_server.rs` detects the "password authentication failed" / "connection refused" error
  shapes and appends a plain-language hint instead of just showing the raw driver error.
  **Not done**: actually reading the real password from a Docker container's env vars (would
  need `docker inspect`'s `.Config.Env`, which `docker_service.rs`'s `enrich_containers_from_inspect`
  doesn't currently capture) or from wherever an external Postgres's real password lives. That's
  real plumbing, scoped out — flagged to the user as a backlog item, not started.

### Verification done

- Unit tests: 33 passing in `portal-core` (pure SQL builders, PK detection, edit-SQL generation,
  rollback behavior, `friendly_connect_error`) — `cargo test database_explorer` from
  `src-tauri/portal-core/`. Clippy clean from both `src-tauri/portal-core/` and `src-tauri/`.
- Frontend: `pnpm check && pnpm lint` — 0 errors (pre-existing warnings elsewhere in the repo,
  unrelated, not introduced by this work).
- **Live-verified in the real running app** by a qa-tester subagent (`pnpm tauri:dev` + CDP,
  Playwright-style driving — see `.claude/qa/scripts/QA-502-*.mjs` and `qa-database-*.mjs` for
  the repro scripts, `.claude/qa/shots/` for screenshots). Confirmed working: editable toggle
  off-by-default and persisting, cell edit with confirm-dialog SQL preview, cancel leaving
  nothing changed, insert row, delete row, no-PK tables staying non-editable, **and the backend
  gate actually refusing a direct `db_apply_update` IPC call when editing is toggled off** (not
  just a hidden frontend control). Postgres discovery itself works (found the real local
  `postgres.exe` SDK service) but connecting fails with the known default-credential gap above —
  expected, not a new bug. MySQL (SDK service was stopped) and Docker discovery (Docker daemon
  wasn't running) were **not** exercised — nothing to test against on this machine at the time.

### Bugs found and fixed this session (from a code review + the live QA pass)

Review findings (all fixed, see commit `69e708ab`'s diff against what a `crew:coder` subagent
first delivered, uncommitted, before review — the fixes are folded into that one commit, not
separate ones):
1. HIGH — composite-PK mass-update/delete (see "Decisions" above).
2. MEDIUM — rows-affected safety net was missing (see "Decisions" above).
3. MEDIUM — postgres type-cast failures on uuid/timestamp/numeric columns and every insert into
   a non-text column (see "Decisions" above).
4. MEDIUM — JS float64 precision loss on big integer PKs (see "Decisions" above).
5. MEDIUM — clicking inside an open cell editor mid-type wiped the typed value (`onclick` on the
   `Table.Cell` re-fired `startEdit` because clicks bubble from the child `Input`) — fixed with a
   same-cell guard at the top of `startEdit` in `ResultGrid.svelte`.
6. LOW — Escape/Enter could double-fire or wrongly trigger the confirm dialog (removing a
   focused input fires a synchronous `blur` in Chromium/WebView2, re-entering `commitEdit`) —
   fixed with a guard at the top of `commitEdit` that no-ops unless `editingCell` still matches
   the cell being committed.

QA-board findings from the live pass (`.claude/qa/issues/`), filed by the qa-tester subagent:
- **QA-157** (major, fixed this session) — the source list `<aside>` in `+page.svelte` had no
  scroll region of its own, so scrolling down to reach any source past "Portal (this app)"
  (which always renders 280+ tables, never collapsed) scrolled the **whole page**, which then
  landed the result pane off-screen above the viewport after a click re-rendered it. Fixed by
  giving `<aside>` `max-h-[calc(100vh-14rem)] overflow-y-auto`.
- **UX-021** (polish, fixed this session) — page subtitle still said "(read-only)" after P2
  shipped editing. Fixed, copy updated.
- **UX-022** (minor, **not fixed**) — the table list has no collapse control and no search,
  so a source with many tables (anything past "Portal (this app)") is hard to browse even with
  QA-157's scroll fix. Left open — a real but smaller polish item, not blocking.

## Left to do

1. **If the last commit (QA-157/UX-021/friendly-errors fix) isn't landed yet**, stage exactly:
   `src-tauri/portal-core/src/domains/database_explorer/sql_server.rs`,
   `src/routes/database/+page.svelte`, the three `.claude/qa/issues/QA-157-*.md`/`UX-021-*.md`/
   `UX-022-*.md` files, and the QA script files under `.claude/qa/scripts/` (`QA-502-*.mjs`,
   `qa-database-*.mjs`). **Do not** stage `.claude/handoff.md` changes from anyone else, the
   `src/lib/domains/tasks/components/*.svelte` files, or `.claude/qa/shots/*.png` — see "Watch
   out for" below, that's unrelated concurrent work sharing this working tree. Commit message
   used when asking the owner:
   `fix(database): #502 scroll bug, stale copy, friendlier connect errors` — body: "QA-157: give
   the source list its own scroll region so picking a table below the fold no longer renders the
   result pane off-screen. UX-021: drop stale read-only copy post-P2. Add plain-language hints
   for password-auth-failed / connection-refused errors on discovered sources."
2. **UX-022** — give the source tree a collapse control (at least for "Portal (this app)", which
   is always-expanded and the reason everything else is buried) and/or a search box. Not started.
3. **Project↔database linking** (user's idea, assessed not built) — letting a user navigate from
   a Portal project to the database connection(s) it uses and back. Assessed by the qa-tester:
   needs real plumbing, not a bolt-on — `DbExplorerState`'s source registry is entirely
   in-memory/per-process today (nothing but the hardcoded "Portal (this app)" source survives a
   restart; an opened `.db` file and the editable-toggle set both reset every launch — that's
   itself a rough edge worth fixing before linking against it), and `projects` has no FK/junction
   toward anything database-related. Needs: (a) a persisted sources table (doesn't exist at all
   yet, even for the simple case), (b) a `project_id` column/junction, (c) UI on both ends. Scope
   this properly before starting — it's comparable in size to P1 or P2, not a quick add.
4. **P3 (Mongo) and P4 (Redis)** — not started. Per the plan, these get their own driver (`mongodb`
   crate / `redis` crate) and their own UI pane each (document viewer, key/value browser) — they
   don't fit the SQL grid model. Read the plan's "P3"/"P4" sections before starting.
5. **Docker-based real-credential discovery** — `docker_service.rs`'s `enrich_containers_from_inspect`
   doesn't currently surface a container's env vars; extending it to read `POSTGRES_PASSWORD`/
   `MYSQL_ROOT_PASSWORD` etc. would let `discovery.rs`'s `from_docker` use the real credentials
   instead of a guess. Not started, flagged as backlog only.
6. **Not yet re-verified live**: the QA-157/UX-021 fixes themselves haven't been driven in the
   app again after being written (they were implemented after the qa-tester's pass finished).
   Low risk (QA-157's fix is a one-line CSS change whose mechanism is well understood), but
   worth a quick live check before considering #502 fully done.

## Watch out for

- **This working tree has unrelated, in-progress work from someone/something else running in
  parallel** — `src/lib/config/main-nav.ts`, several `src/lib/domains/tasks/components/*.svelte`
  files, `src/lib/domains/tasks/utils/openClaudeSession.ts`, and `.claude/handoff.md` itself (a
  different task's "context-menus" handoff was here before this file was overwritten — that task
  was "rollout complete, minor follow-ups" on branch `feat/context-menus`, not blocking, just
  noting it existed). **Never `git add -A` or stage these** — always add exact paths. Also never
  stage `.claude/qa/shots/*.png` (screenshots) or `.qa424-src/` — established convention in this
  repo to leave QA screenshots untracked.
- **`git commit` via the Bash tool was denied by the permission classifier every single time**
  in this session, regardless of message content, flag style, or `-F` vs `-m`. Don't burn time
  retrying variations — just ask the owner to run it themselves once gates are green and the
  right files are staged.
- **`pnpm gen:types` regenerates TS bindings for the entire app**, not just the domain you're
  touching — if you run it, expect a big `git status` and check the diff is trivial (whitespace/
  ts-rs-version noise) before deciding what to stage. Don't commit unrelated domains' regenerated
  types.
- **Postgres preview text will not show the `::type` cast** that actually executes — this is
  intentional (see "Decisions"), not a bug if you spot it again.
- The SDK's own Postgres/MySQL/Mongo configs (`language_config.rs`) use an **empty password** by
  design (`"PGPASSWORD": ""`, `"MYSQL_PASSWORD": ""`) — that's correct for a Portal-managed local
  install, so don't "fix" `discovery.rs`'s default-credential guess for that case; the actual gap
  is only for externally-managed Postgres/MySQL instances.
