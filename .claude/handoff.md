# STATUS 2026-11-25 (later): sync P1 BUILT, not deployed

- Desktop: commit 73037061 (feat/agent-platform-sidecar). Server: portal_server `feat/sync-snapshot` (1074295, 1e0cb20). Nothing pushed/deployed.
- Plan doc rewritten (docs/architecture/online-sync-plan.md). UI at /settings/sync.
- USER TODO: deploy server (Cloud Run), paste server URL + token in sync settings (no desktop->server login exists), unlock vault with password once after each restore.
- Not run end to end (no real server/Mongo/R2). First real push/pull is the test. Auto-resolve of project paths only keeps same-path-exists; no git-remote matching.
- Still open: portal_os backup (big-ISO history), per-row multi-device (deferred).

---

# Handoff — holiday backup (done) + online-sync design (ready to build)

_Written 2026-11-25 for a fresh session with no memory of the prior conversation._

## Goal

(1) Back everything up to GitHub before the user's holiday — **DONE**. (2) Design + build **quick, single-device** online sync for portal_desktop so the user can move their data to another machine — **DESIGNED, ready to build (not started).**

---

## Thread 1 — Backup (DONE)

Pushed to GitHub: portal_desktop (`feat/agent-platform-sidecar`, HEAD has the sync plan doc), pocket_portal (3 branches), portal-remote (main+2), portal-media (1), portal-fitness (main + 462-file WIP, `mongodb_data/`+`*.tsbuildinfo` gitignored). New PRIVATE repos created+pushed: `portal_sdk`, `portal_plexus`, `ui-template`.

- **portal_os — NOT backed up** (left local): history has a 2.05 GB ISO + a 244 MB ISO > GitHub's 100 MB limit. Empty `tanvoid0/portal_os` repo exists. To fix: `git filter-repo --strip-blobs-bigger-than 100M` then push.
- **.env** = 2 Tauri signing keys, already GitHub Actions secrets (write-only). portal_desktop repo is PUBLIC so .env NOT pushed. User to copy the 2 values to their password manager.
- Gotcha: portal_os/portal_plexus/ui-template `.git` owned by a different Windows SID + had stale Aug-2026 `index.lock`s → use `git -c safe.directory=<ABSPATH>`; locks removed.

---

## Thread 2 — Sync (FINAL DESIGN — build this)

**Server:** `D:/projects/production/portal/pocket_portal/server/portal_server` (NestJS+Mongo, Cloud Run, Cloudflare R2). **Plan doc:** `docs/architecture/online-sync-plan.md` (describes the older per-row design — REWRITE to the snapshot design below).

### User's final requirements

- **No multi-device now** (maybe future). **Single device at a time.**
- **Quick** — user is switching off the desktop onto another machine imminently.
- **Must SHOW conflicts + timestamps** so the right data is synced (PS+-style "cloud save is newer — keep which?").

### THE DESIGN: whole-file encrypted snapshot → R2, metadata record in Mongo

Chosen over per-row because the user is single-device + wants it quick; chosen file-sync because it matches "server just stores and syncs files", and the DB carrying the `vault` row makes credentials portable by password for free. Per-row is deferred to a future multi-device phase (additive; see "Future").

**Server (new snapshot module, R2-backed):**

- Encrypted DB snapshot **file → Cloudflare R2** (server already has private R2: `src/services/media/private-r2.storage.ts`, used by trips/finance docs).
- **Metadata record in Mongo** per snapshot: `{ user_id, version (monotonic int), device_id, client_updated_at (ISO), size, content_hash, r2_key, created_at }`. Owner key `user_id` (so `src/common/users/account-deletion.service.ts` sweep covers the record — but **R2 objects need explicit delete**, the sweep is Mongo-only; add R2 cleanup on account delete + prune old versions).
- Endpoints (auth = `@UseGuards(CommonAuthGuard)` + `@UserId()`, both global): `POST /api/sync/snapshot` (upload to R2 + create record, version++), `GET /api/sync/snapshot/latest` (record + download), `GET /api/sync/snapshot/versions` (list — powers the timestamp/conflict UI).
- **Conflict detection:** client sends the `base_version` it last pulled. If server's current version > base_version → **conflict** (another machine uploaded since) → 409 + the current record. Client shows both timestamps/sizes and lets the user pick keep-local (force upload, version++) or take-cloud (download).

**Desktop client P1:**

- Backup: SQLite `VACUUM INTO` a temp copy (drop volatile/log tables — live DB is 24 MB, mostly logs) → encrypt with a **password-derived key** (PBKDF2, reuse `pocket_portal/packages/portal_crypto` format) → upload to R2 via the server → store new version+timestamp locally.
- Restore: `GET latest` → show its timestamp vs local → if cloud newer than local base, prompt → download → decrypt → verify `content_hash` → **atomic swap** (close the SeaORM pool, replace the file, reopen) → **re-run project discovery** (machine-local fields `path`/`git_*`/`dev_port`/`discovery_path` point at the old machine's disk; discovery re-derives them).
- Conflict UI: local-vs-cloud timestamps + sizes, "Keep this machine / Take cloud".
- E2EE: password-derived key over the whole file → server + R2 see ciphertext only. The snapshot carries the `vault` row → credentials unlock by password on the new machine (no keychain needed — `unlock_with_password` re-derives MK from the DB row).

**Build notes:** build + commit in portal_server, **do NOT push or deploy** (Cloud Run deploy stays the user's call). The old ~80%-built per-row module was **stashed** in portal_server (`git stash list` → "wip per-row sync module (superseded…)") — ignore it for snapshot P1; recover only if per-row is revived.

### Future (not now): per-row multi-device

When concurrent multi-device editing or a 2nd client (web/mobile) is real, add an **additive** generic per-row document store (per-user/namespace/doc_id, push/pull + cursor + tombstones — like pocket_portal's apps). Needs client-side outbox + SQLite triggers + `sync_map` (autoincrement PKs collide) + a separate portable DEK (the vault master key is keychain-bound, NOT password-derived). Full spec in the stashed module + `docs/architecture/online-sync-plan.md`.

### Grounded facts (from a 3-agent code study)

- **Server:** reuse `src/shared/sync/sync-batch.ts`; NoteSync at `src/apps/task/modules/notes/*` is the module template; `MongoRepository` base at `src/shared/mongoose/mongo.repository.ts`; mount via 1 line in `src/app.module.ts`; global prefix `api`; default body limit 100 kb (raise route-scoped); R2 at `src/services/media/private-r2.storage.ts`.
- **Desktop:** migrations append to `get_migrations()` in `src-tauri/portal-core/src/migrations/mod.rs`; periodic-job precedent `spawn_expiry_sweep` at `src-tauri/src/daemon/pairing.rs:264`, spawned from `src-tauri/src/lib.rs` ~171; local DB `%APPDATA%\com.tanvoid0.portal-desktop\portal_desktop.db` ≈ 24 MB (dev uses the `.dev` dir); separate `disk_utility.db` ≈ 3 MB.
- **Vault:** master key is random bytes in the OS keychain; password only _wraps_ it into the `vault` table row (`pw_wrapped_mk`, PBKDF2 600k). `unlock_with_password` re-derives MK from that row **without the keychain** — this is why a copied/synced DB decrypts by password. `.env` files are NOT stored by the app (read live from disk).

---

## IMMEDIATE STOPGAP (tell the user — the sync feature isn't built yet)

To move data to the new machine NOW, before any sync code exists:

1. Close Portal Desktop on both machines.
2. Copy the release data dir `%APPDATA%\com.tanvoid0.portal-desktop\` (contains `portal_desktop.db` + `disk_utility.db`) to the same path on the new machine.
3. Open Portal there, **unlock the vault with the password** (the DB carries `pw_wrapped_mk`, so it works without the old machine's keychain).
4. Re-run project discovery (project paths pointed at the old machine).
   **Verify the vault unlocks on the new machine BEFORE wiping the old one.** This is the manual version of the restore flow above.

## Gotchas

- `| tail` on `git push` masks git's exit code (a 500'd push reported exit 0) — verify with `git ls-remote`.
- portal_desktop pre-push husky hook = `pnpm check` + `pnpm lint` + 2× `cargo check`; `prettier --check` covers `docs/` → run `pnpm exec prettier --write` on new `.md` before pushing.
- NO Claude attribution in commits/PRs (user global rule). crew:coder may auto-commit — brief it not to.
- Cap: never more than 10 agents at once.

## First moves next session

1. Give the user the IMMEDIATE STOPGAP above if they still need to move data.
2. Rewrite `docs/architecture/online-sync-plan.md` to the snapshot design.
3. Build P1: server snapshot module (R2 + metadata + conflict 409) → desktop backup/restore + conflict UI. Commit in portal_server; do NOT push/deploy.
