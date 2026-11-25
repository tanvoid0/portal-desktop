# Online Sync — Portal Desktop ↔ portal_server

**Status:** plan / not built · **Date:** 2026-11-25
**Server:** `D:/projects/production/portal/pocket_portal/server/portal_server` (NestJS + MongoDB, Cloud Run)
**Goal:** keep working fully offline on local SQLite (today's behaviour), and sync that data online so another machine can restore it — with secrets locked so only the right user, with the right vault password, can decrypt them.

---

## 1. The core decision

The server is a **dumb, end-to-end-encrypted document store**. It stores and ships opaque ciphertext blobs keyed by user + namespace + doc id, and **never parses them**. The desktop app keeps SQLite as the source of truth and owns every query — exactly as it does now. Sync is purely additive.

This is what was asked ("the server just stores and syncs files; our app deals with the database and queries"), and it falls out cleanly:

- **One server module, forever.** No per-domain server work as the app grows — a new synced table is a client-side change only.
- **Server breach leaks nothing.** It holds ciphertext and a password-wrapped key bundle; without the vault password there is nothing to read.
- **Per-environment secrets are free** — they're just another namespaced collection of the same blobs.

Rejected alternative: typed per-domain server schemas (the pocket_portal mobile pattern). It gives server-side queries and a future web client that reads the data, at the cost of per-domain server work and the server reading plaintext. We don't want server-side queries (the desktop owns them), so we don't pay for them. Upgrade path stays open if a web client is ever needed.

```
   Desktop A                      portal_server (Cloud Run)              Desktop B
┌──────────────┐   push blobs   ┌───────────────────────────┐   pull   ┌──────────────┐
│ SQLite (SoT) │ ─────────────▶ │  sync_blobs  (ciphertext) │ ───────▶ │ SQLite (SoT) │
│ sync_outbox  │                │  + wrapped DEK bundle      │          │ apply + map  │
│ vault / DEK  │ ◀───────────── │  server never decrypts     │ ◀─────── │ vault / DEK  │
└──────────────┘   pull blobs   └───────────────────────────┘   push   └──────────────┘
```

---

## 2. Server side — one new module

The server already has the whole sync spine. We copy the **NoteSync** module and swap typed fields for one opaque blob.

**Reuse (do not rebuild):**

- `src/shared/sync/sync-batch.ts` — `applySyncBatch(ops, apply) → {applied, rejected}` and `nextPullCursor(now, pages) → ISO` (5 s overlap window). The entire push/cursor engine.
- `src/apps/task/modules/notes/note-sync.service.ts` (lines 45–83) — the cleanest pull template: two queries per collection (live `updated_at > since, deleted_at: null`; tombstones `deleted_at > since`), returns `{cursor, …, deleted_ids}`.
- `src/common/auth/common-auth.guard.ts` + `src/common/auth/user-id.decorator.ts` — `@UseGuards(CommonAuthGuard)` + `@UserId() userId`. Both `@Global`; no import needed.
- `src/shared/mongoose/mongo.repository.ts` — `MongoRepository<T>` with owner scoping and `softDelete` (tombstones).
- `src/common/users/account-deletion.service.ts` — auto-wipes any collection with a top-level `user_id`. **Use `user_id` as the owner key and GDPR deletion is free.**

**Add (`src/apps/sync/`):**

| file                          | content                                                                                                                                                                                                                                                                                                                                                                                       |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `schemas/sync-blob.schema.ts` | `@Schema({collection:'sync_blobs'})`; `_id` String UUID (client-supplied `doc_id`); `user_id` (indexed), `namespace` (e.g. `desktop:local` / `desktop:prod`), `kind` (logical table), `ciphertext` (String, base64), `schema_version`/`key_id` (String, opaque), `created_at`, `updated_at`, `deleted_at`. Compound indexes `{user_id:1, namespace:1, updated_at:1}` and `{…, deleted_at:1}`. |
| `sync-blob.repository.ts`     | one-line subclass of `MongoRepository`, `@InjectModel(…, PORTAL_DB_CONNECTION)`, owner key `user_id`.                                                                                                                                                                                                                                                                                         |
| `dto/blob-sync-push.dto.ts`   | `{operations: [{id, action:'put'\|'delete', doc_id, namespace, kind?, ciphertext?, key_id?, client_updated_at?}]}`. `ciphertext` is `@IsString() @MaxLength(N)` — **never `@IsObject`/`@ValidateNested`**, so the server can't parse it. `@ArrayMaxSize` on `operations`.                                                                                                                     |
| `blob-sync.service.ts`        | `push` = `applySyncBatch(ops, applyOp)` with last-writer-wins on `client_updated_at` vs stored `updated_at`, `softDelete` for tombstones, delete-of-missing returns `true`. `pull` = the two-query pattern + `nextPullCursor`, returns `{cursor, docs, deleted_ids}`. Every query carries `user_id: userId` — a foreign `doc_id` resolves as not-found.                                       |
| `blob-sync.controller.ts`     | `@Controller('sync')` `@UseGuards(CommonAuthGuard)`; `GET /api/sync?namespace=&since=<iso>`, `POST /api/sync/push`.                                                                                                                                                                                                                                                                           |
| `sync.module.ts`              | `MongooseModule.forFeature([...], PORTAL_DB_CONNECTION)`, lists controller/repo/service.                                                                                                                                                                                                                                                                                                      |

**One line in `src/app.module.ts`** (imports array, next to `TaskModule`).

**Must-fix server risks (from the study):**

- **Body limit.** No override today → Express default **100 kb** JSON. Blobs above that 413. Raise it **route-scoped** on the sync push route (not globally — global widens every endpoint's attack surface).
- **Cursor tie-break.** `nextPullCursor` can resend a page forever if > 5000 rows share one millisecond (bulk import). Give each op a distinct, monotonic `updated_at`, or add an `_id` tie-break.
- **Quota/abuse.** No per-user size or op-count cap exists on any sync endpoint. Add `@ArrayMaxSize`, `@MaxLength`, and consider a per-user document cap.
- **ValidationPipe is `forbidNonWhitelisted`.** Any client field not in the DTO → 400 on the whole batch. Keep DTO and client model in lockstep.
- **KDF iteration floor.** The server holds the wrapped-key bundle; a client must enforce a minimum iteration count so a tampered stored value can't weaken the KDF.

---

## 3. Desktop side — change tracking + sync engine

### The two hard facts

1. **Primary keys are mixed.** `credentials` use String uuid (collision-safe). `projects`, `tasks`, `documents`, `custom_scripts`, project-env tables use **autoincrement i32/i64 → they collide across devices.**
2. **There is no soft-delete anywhere** (all hard deletes) and **`updated_at` is unreliable** (not bumped on several write paths; `project create` sets it `None`). There is **no single write choke point** — repositories, raw SQL, discovery import, the mover, and the daemon/MCP board all write.

### The answer: SQLite triggers → an outbox (one migration, no table ALTERs)

Add one migration (append to `get_migrations()` in `src-tauri/portal-core/src/migrations/mod.rs`, after `createProjectEnvFilesTable`) that creates:

- **`sync_outbox(seq INTEGER PK AUTOINCREMENT, entity TEXT, row_id TEXT, op TEXT, ts)`** — the durable change log.
- **`sync_state(key, value)`** — pull cursor, this install's `device_id`, and an `applying` flag.
- **`sync_map(entity, local_id, sync_id TEXT)`** — local autoincrement id ↔ stable uuid. Credentials and other String-PK tables use their own id as the sync id.
- **`AFTER INSERT/UPDATE/DELETE` triggers** on each synced table that insert into `sync_outbox`.

Why triggers beat hooking Rust code: they sit **below every write path including raw SQL**, so nothing can forget to log a change. The `DELETE` trigger captures a tombstone into the outbox, so **no `deleted_at` column is needed** on existing tables — the hard-delete code stays untouched. (Template for raw-SQL migrations: `manager.get_connection().execute_unprepared(...)`, already used in `m20261119_000051_unify_runs.rs`.)

**Echo suppression:** when applying pulled remote changes, set `sync_state.applying=1`; the triggers carry a `WHEN (SELECT value FROM sync_state WHERE key='applying')='0'` guard so applied writes aren't re-queued.

### The sync engine (Rust)

Copy the existing periodic-job precedent **`spawn_expiry_sweep`** (`src-tauri/src/daemon/pairing.rs:264` — `tauri::async_runtime::spawn` + `tokio::time::interval`), spawned once from `src-tauri/src/lib.rs` setup (next to line 171). Each tick, if online and the vault is unlocked:

1. **Pull** `since = sync_state.cursor` → decrypt each blob with the sync DEK → translate sync_ids → upsert into SQLite under `applying=1` → advance cursor.
2. **Push** outbox rows (`seq > last_pushed`) → serialize row to JSON → translate local ids to sync_ids → encrypt → `POST /api/sync/push`. On `applied` advance; on `rejected` (server newer) pull that doc, resolve, retry.

### Scope of the first synced set

**In:** `credentials` (already ciphertext), `custom_scripts`, `projects` (minus machine-local columns), `documents`, `tasks`.
**Out:** `device_approvals` (per-machine LAN state), `vault` (key distribution — separate decision), agent/coder/AI logs, execution history, sdk + lookup tables.

**Machine-local fields must not overwrite the other device's disk view.** On `projects`, exclude `path`, `git_*`, `size`, `file_count`, `dev_port`, `regenerable_dirs`, `discovery_path` from the shared blob (keep them per-device). `discovery_path` (`<root>/<category>/<name>`) is the natural cross-device _identity_ for a project but is itself device-relative — map on identity, don't sync as data.

**Cross-references inside blobs** (`tasks.parent_id`, `tasks.blocked_by/blocks` JSON arrays, `projects.parent_id`) hold integer ids — translate to sync_ids on push and back on pull.

**Gotchas:** `decrypt_credential` writes `last_used` on every read → exclude `last_used` from change detection or it floods the outbox. Dev builds copy the release DB on first launch (`database.rs:70-90`) → regenerate `device_id` for dev so it isn't seen as the same device.

---

## 4. Crypto — portable E2EE (the critical constraint)

**The blocker the study found:** the vault **master key (MK) is NOT derived from the password.** It's 32 random bytes from `OsRng`, stored raw in the **OS keychain**; the password only _wraps_ it (PBKDF2 600k → AES-GCM → `pw_wrapped_mk` in the local `vault` row). So **a new machine cannot decrypt anything from the password alone** — MK is machine-bound.

**The fix: a separate, portable sync DEK.**

1. Generate a random **sync DEK** (independent of the keychain MK — rotating it never re-encrypts the local credentials table).
2. Wrap it exactly as pocket_portal does (`packages/portal_crypto/lib/src/key_wrap.dart`): `WrappedKeyBundle{kind:'password', salt, iterations, wrapped_dek}`, KEK = PBKDF2-HMAC-SHA256(password, salt, iterations), AES-256-GCM, AAD `portal-dek-wrap-v1`. Also store a **recovery-key wrap** (HKDF, distinct info label) so a forgotten password doesn't lose the synced data.
3. Store only the bundle + ciphertext on the server.
4. **Encrypt each synced blob** with the DEK in pocket's `PortalVaultBlob` format (`nonce||ct||tag`, AAD `portal-vault-v1`). All primitives (AES-256-GCM, PBKDF2-SHA256, HKDF-SHA256) are already in the desktop via `ring` — this is new glue, not new crypto. **Do not** change the local `EncryptionResult` format (different AAD/layout; local-only).

**New-machine restore:** install → sign in to the server (identity) → enter vault password → download bundle → derive KEK → unwrap DEK → pull → decrypt. Two independent secrets (server login ≠ vault password); a server breach alone reveals nothing.

**Credentials across devices:** don't try to port MK. Sync credential rows by re-encrypting their _plaintext_ under the **sync DEK** inside the blob; the receiving device re-encrypts under its own local MK on import.

---

## 5. Per-environment `.env` secrets

**Today `.env` files are not stored in the app** — they're read live from disk each call (`domains/projects/env/service.rs`), and only `project_env_links` (links to vault credentials) are persisted. So there is **no at-rest store** to sync.

Plan: give each environment's secrets a namespace (`desktop:local`, `desktop:prod`, …) and a `kind='env_secret'` collection. To sync a raw `.env`, **import it into the encrypted store first** (into the credentials table, or a small dedicated `env_secrets` table), then it syncs as DEK-encrypted blobs like everything else. On another machine: unlock vault → pull → decrypt → the env values are available, server never sees plaintext.

---

## 6. Conflict resolution

**Last-writer-wins by timestamp**, matching the existing server contract (`note-sync` `isConflict`). It's the lazy, parity-preserving choice for single-user multi-device.

`// ponytail: LWW by updated_at — known ceiling: two offline devices editing the same row, last push wins, other edit lost. Upgrade to per-field merge or a server compare-and-set version only if this bites.` Because `updated_at` isn't reliably bumped locally, the **outbox row's timestamp** is the logical clock for conflict resolution, not the table's `updated_at`.

---

## 7. Phasing

- **P1 — server module.** Copy NoteSync → `src/apps/sync/`, opaque blob, route-scoped body limit, quota caps. Deploy to Cloud Run. _(~5 files + 1 line; no desktop changes.)_
- **P2 — desktop plumbing.** One migration: `sync_outbox` + `sync_state` + `sync_map` + triggers. Sync engine (`spawn_expiry_sweep` clone). Sync DEK generate + password/recovery wrap. **One domain end-to-end first** (`custom_scripts` — simplest, String-free, low risk).
- **P3 — crypto portability + restore.** New-machine flow: sign-in → password → unwrap DEK → pull → decrypt. Credential re-encryption under the DEK.
- **P4 — roll out remaining domains + env secrets.** Each is "register a `kind` + add triggers + id-map" — **no new server work.**

---

## 8. Decisions needed from you

Each has a recommendation; confirm or redirect.

1. **Conflict policy** — LWW by timestamp (recommended, matches the server) vs server compare-and-set version. → **LWW.**
2. **Primary keys** — `sync_map` side table (recommended, tiny) vs a full i32→uuid migration (invasive: touches `parent_id`, JSON id arrays, junction tables, TS types, MCP). → **sync_map.**
3. **Credential sync in the first cut?** If yes, use the sync-DEK re-encryption above (not MK export). → **recommend deferring credentials to P3**, prove the pipeline on `custom_scripts` first.
4. **`.env` secrets** — import into an encrypted store before syncing. Reuse the `credentials` table, or add a dedicated `env_secrets` table? → **recommend a dedicated `env_secrets` table** (keeps project env separate from user credentials).
5. **Auth/identity** — the desktop needs a server login (the server's existing Google / phone-OTP platform auth) to get a `user_id` namespace. Confirm the desktop should adopt that auth, separate from the vault password.
6. **Dev vs release** — dev builds sync too, or release only? (Dev copies the release DB, so it needs a regenerated `device_id` either way.) → **recommend release-only sync** at first.

---

## 9. Risk ledger (carry into build)

| area    | risk                                                   | mitigation                                              |
| ------- | ------------------------------------------------------ | ------------------------------------------------------- |
| server  | 100 kb body limit → 413 on blobs                       | route-scoped parser limit                               |
| server  | cursor resends page if >5000 rows/ms                   | monotonic `updated_at` + `_id` tie-break                |
| server  | no quota/rate limit on sync                            | `@ArrayMaxSize`, `@MaxLength`, per-user doc cap         |
| desktop | autoincrement PK collisions                            | `sync_map` + id translation in blobs                    |
| desktop | no soft-delete, unreliable `updated_at`                | outbox triggers capture tombstones + logical clock      |
| desktop | machine-local fields corrupt other device              | exclude `path`/`git_*`/`dev_port`/… from shared blob    |
| desktop | `last_used` write floods outbox                        | exclude from change detection                           |
| crypto  | MK keychain-bound, not portable                        | separate password-wrapped sync DEK                      |
| crypto  | KDF params differ (600k/32B vs 210k/16B)               | honour `iterations`/`salt` from bundle; enforce a floor |
| crypto  | E2EE strength bounded by password (min 8, PBKDF2 only) | raise minimum, fold recovery key into KEK               |

---

_Grounded by a three-way code study of `portal_server`, `portal-core` (desktop Rust), and the vault/crypto layer. File:line anchors above are from that study and should be re-checked at build time._
