# Online Sync — Portal Desktop ↔ portal_server

Status: P1 (snapshot) built 2026-11-25. Per-row multi-device sync is deferred.

## 1. Decision

Single device at a time, quick to ship, conflicts visible. So: **whole-file encrypted
snapshot**, not per-row sync.

- The desktop DB is one SQLite file. A snapshot of it is encrypted on the client with a
  key derived from a user password, uploaded to private Cloudflare R2 through
  `portal_server`. The server and R2 only ever see ciphertext.
- A metadata record in Mongo per snapshot gives version + timestamps, which powers the
  "cloud is newer — keep which?" prompt.
- The `vault` table row travels inside the DB. `unlock_with_password` re-derives the
  master key from that row without the OS keychain, so credentials unlock on a new
  machine by password alone.

Per-row was rejected for P1: it needs an outbox, SQLite triggers, an id map (autoincrement
PKs collide across machines) and a separate portable DEK. None of that is needed for one
device at a time.

## 2. Server (`portal_server`, module `sync`)

Mongo `sync_snapshots`: `{ user_id, version, device_id, client_updated_at, size,
content_hash, r2_key, created_at }`, unique on `(user_id, version)`. Bytes live in
`PrivateR2Storage` under `sync/<user_id>/<version>.bin`. Newest 5 versions are kept;
account deletion also deletes the R2 objects (the Mongo sweep alone would orphan them).

| Endpoint                                   | Behaviour                                                                                                                                                                                                                  |
| ------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `POST /api/sync/snapshot`                  | raw bytes; query `base_version, device_id, client_updated_at, content_hash, force`. Verifies the hash. If `current > base_version` and not `force` → `409 {code:'conflict', current}`. Else stores, `version = current+1`. |
| `GET /api/sync/snapshot/latest`            | latest record, 404 if none                                                                                                                                                                                                 |
| `GET /api/sync/snapshot/versions`          | `{versions}` newest first                                                                                                                                                                                                  |
| `GET /api/sync/snapshot/:version/download` | ciphertext bytes, owner-checked                                                                                                                                                                                            |

Upload cap is 32 MB (Cloud Run request limit). Snapshots are trimmed (log tables dropped)
and gzipped before encryption to stay well under it.

## 3. Desktop

**Backup**: `VACUUM INTO` temp copy → drop volatile/log tables → gzip → AES-256-GCM with a
PBKDF2-HMAC-SHA256 key (600k) → `POST`. Records `base_version` and `last_synced_at`
locally. The password is never stored.

**Restore**: `GET` → verify `content_hash` → decrypt → integrity-check → keep the old DB
as `.pre-restore.bak` → swap the file → re-run project discovery (project
`path`/`git_*`/`dev_port`/`discovery_path` point at the old machine's disk).

**Conflict UI**: on 409 the client shows this machine vs cloud (timestamps, sizes,
device) and offers _Keep this machine_ (force push) or _Take cloud_ (pull).

## 3b. Per-device paths

Project folder paths (`path`, `git_*`, `dev_port`, `discovery_path`) differ per PC. They
are kept per device in `project_device_paths (project_id, device_id, …)`, which travels
inside the snapshot with every device's rows intact.

- Push upserts this device's row. Restore re-applies this device's row if present, so
  syncing back to the original machine resolves exactly as before.
- No row for this device: try auto-resolve via discovery, else mark the project
  unresolved. The foreign path is never treated as valid and the project is never deleted.
- The user can _Locate folder_ or _Ignore_. Two PCs resolve independently and never
  overwrite each other's rows.
- Restore keeps this machine's own `device_id` / server URL rather than the snapshot's.

## 4. Not in scope

- Multi-device concurrent editing. When it becomes real, add an additive per-row document
  store (push/pull + cursor + tombstones), plus a portable DEK — the vault master key is
  keychain-bound, not password-derived.
- `.env` files are not stored by the app; they are read from disk, so they do not sync.
- `disk_utility.db` (separate, ~3 MB) is not part of the snapshot.

## 5. Risks

- Lose the snapshot password → backup is unreadable. The UI warns on first backup.
- Cloud Run deploy of the server is a manual step and is not done by this change.
- Restore replaces local data wholesale; the `.pre-restore.bak` is the only undo.
