# Project env files in the vault (bulk backup / restore)

Status: backup/restore built 2026-11-25. "Sign in with GitHub" (section 7) is a plan only.

## 1. Why

Env files must never go to GitHub, not even encrypted. Portal already keeps secrets in the
vault: AES-256-GCM under one random master key, wrapped by the master password (PBKDF2), a
recovery key, and the OS keychain. The `credentials` rows travel inside the sync snapshot
(`online-sync-plan.md`), so a vault entry reaches a new machine with one password.

What was missing is the _contents_ of `.env*` files. The env tab only links single keys to
credentials, and `online-sync-plan.md` §4 says env files do not sync. This adds bulk
backup (disk to vault) and restore (vault to disk).

## 2. Model

One `.env*` file = one credential row. No new table and no migration (the installed release
refuses migrations it does not know; this feature adds none).

- `credential_type = "env_file"`, `name = "<project>/<rel path>"`, `tags = ["env-file"]`.
- `value` = the file verbatim (comments, quoting, CRLF, BOM), encrypted by `CredentialService`
  like any credential. No new crypto.
- Link to the project = plaintext `metadata`:
  `{kind:"env-file", project_id, project_name, git_remote, rel_path}`.
  No hashes, key names or values in plaintext. "Same content?" is answered by decrypting.
- `rel_path` is `/`-separated and relative to the project root (`backend/.env`).

Code: `portal-core/src/domains/projects/env/backup.rs`, next to `ProjectEnvService` (which
already depends on `credentials`; the reverse would be a cycle). Commands are `local_only`
(IPC and loopback, never LAN) in `src-tauri/src/domains/projects/env_commands.rs`:
`env_backup_scan`, `env_backup_import`, `env_backup_restore`.

## 3. Backup (import)

- Candidates: every project whose `projects.path` is non-empty and exists on this machine
  (`project_device_paths` is what resolves that path per device). Walk up to 4 levels for
  `.env` and `.env.*`, minus templates (`.example .sample .template .dist .tpl .defaults`,
  which live in git). Pruned: `node_modules`, `.git`, `target`, `build`, `dist`, `venv`,
  `.venv`, `.dart_tool`, `Pods`, `.claude`, and the like; also any folder that is itself
  another Portal project. Over 1 MiB or not UTF-8 text (PowerShell 5 `>` writes UTF-16) cannot be a credential value: the scan lists them as not backed up instead of hiding them.
- Per file, matched by `(project_id, rel_path)`: no row = create, same content = unchanged,
  different = `conflict`: the vault copy is kept unless that file is sent with `overwrite`. The UI asks per file and says which side is newer. There is no history (the server keeps 5 snapshots), so a replaced vault copy is gone.
- The vault key is required once, up front. Locked = the normal unlock prompt; cancelled =
  `Vault is locked` and nothing is written. It never loops one prompt per file.
- Reads the disk, never changes it.

## 4. Restore

- The row's project is resolved by `project_id` only when its name or git remote still
  agrees (`projects.id` is AUTOINCREMENT, never reused), else by a unique git remote.
  A name alone never matches: `web` is a common folder name, and a wrong guess puts a secret in the wrong project. No match = skipped and listed as unmatched.
- Target = project path + `rel_path`. `rel_path` must be plain relative components ending in
  an env file name (no `:`, no trailing dot or space: NTFS streams and trimmed names), the target is never a symlink, and the parent folder must resolve inside the project root, so a tampered
  row cannot write anywhere else. The project folder and the file's parent folder must
  already exist: restore creates files, never folders. Honours the project move lock.
- Missing target: created with `create_new` (cannot clobber a file that appears meanwhile).
  Same content: no-op. Different content: `conflict`, nothing written. The caller repeats
  with `overwrite = true` for exactly that row after the user confirms; the replace is a
  uniquely named temp file (`create_new`) plus rename. No `.bak` copy is left, since a stray secret copy is worse than the
  confirmation.
- A restore is a real credential use, so `last_used` is bumped. Scan and comparison are not.

## 5. UI (Settings > Vault > Project env files)

One row per file: new / in sync / differs / only in vault / unknown (vault locked).
Actions: Back up and Restore (each asks before replacing a differing copy on the other side), bulk "Back up new files"
and "Restore missing" (neither can replace anything). Opening the page never prompts: scan uses `peek_value`, so a locked
vault shows `unknown` and an Unlock button.

## 6. Moving the local gpg bundles in (`D:\projects\production\env-backup`)

Portal never touches gpg or the passphrase. The user decrypts:

1. In that folder run its own `restore.ps1 -PassphraseFile <your file> -Apply`. It writes the
   env files into the project folders (existing files skipped) and holds the passphrase only
   in that script.
2. In Portal, Settings > Vault > Project env files > "Back up new files".
3. Check the rows read "in sync", then delete the local bundle repo if wanted.

Only `.env*` files are imported. The bundles also hold keystores, service-account JSON and
the like; a later step can reuse the same row shape with `encoding: "base64"`.

## 7. Plan: Sign in with GitHub (not built)

What exists: Settings > GitHub runs the OAuth device flow (`github_start_device_flow`,
`github_poll_device_flow`), stores the token as a vault credential of type `github_token`
plus a one-row `github_connection`, and reads the OAuth App client id from settings
(`integrations.github.clientId`). The token rides in the snapshot like any credential.

Gaps and steps, smallest first:

1. **Zero config.** Ship a default client id for a Portal OAuth App (device flow needs no
   secret) and keep the settings field as an override. Add a "Sign in with GitHub" button
   to the first-run and Settings > Vault screens.
2. **Unlock first.** `persist_connection` needs the key. Call `vaultGate.ensureUnlocked`
   before the device code is shown, so the user is not stranded after authorising on GitHub.
3. **New machine.** After a sync restore the token is already in the vault. On unlock, call
   `GET /user`; connected if it answers, otherwise re-run the device flow. No second login.
4. **Link repos.** Project to repo links exist (`github_link_existing_repository`,
   `github_detect_local_repository`); show the signed-in login in the project header.
5. **"GitHub signing" is undecided.** If it means commit signing, that is a separate feature:
   keep the signing key as a vault credential, set `user.signingkey` and `gpg.format`, and
   upload the public key through the API. Ask before building it.
6. **Tests.** Mock the GitHub endpoints; no real tokens in tests or fixtures.

## 8. Limits

- Plaintext in the DB: project names and env file paths (credential name and metadata).
- No history, no delete propagation (removing a project leaves its rows; they show as
  unmatched), no nested-project dedup beyond skipping other projects' folders.
- Non-UTF-8 files and files over 1 MiB are not backed up (listed by the scan).
- A project that is deleted and re-added matches its old rows again only by git remote; without one they stay unmatched (back up again).
- File names compare case-sensitively: `.ENV` and `.env` are two rows.

## 9. Run it

The vault is unlocked by the user (Windows Hello or the master password). Agents and tests
never unlock the real vault.

1. Build and start a Portal that contains this branch. `pnpm tauri:dev` is a separate dev
   instance: its own `.dev` data dir (a copy of the release DB made on first launch) and
   daemon on :18421, so what it backs up stays in that copy. Entries only reach the real
   vault from a release build that includes this branch. There is no migration, but
   running any newer build on the real DB applies the migrations that build ships.
2. Settings > Vault: unlock. (Opening the page never prompts; it shows "Vault locked" rows
   until you do.)
3. Project env files: filter if you like, then **Back up new files**. Rows turn "In sync".
4. On another machine: restore the sync snapshot, locate the project folders (Settings > Sync),
   unlock, then **Restore missing**. A file that already exists and differs is only replaced
   after a per-file confirmation.

Tests (throwaway in-memory DB and temp folders, a fixed test key, no keychain):
`cd src-tauri/portal-core && cargo test backup`.
