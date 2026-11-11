# Handoff — phone (wifi adb) QA sweep: Devices pairing + Agents Run, verification interrupted

Written 2026-11-11, for someone with **no memory** of this session. Scoped to phone/Devices/
Agents-Run QA only — does **not** supersede `.claude/handoff.md` (a different, concurrent
session's handoff for the broader `agents` domain P1 build). That session explicitly listed
this session's files (`src/app.css`, `PairDeviceCard.svelte`, `devices/+page.svelte`,
`invokeClient.ts`, QA-109/113/114/115) as "not theirs, don't touch" — this file is the other
side of that boundary. **Read `.claude/handoff.md` too** before touching anything under
`src-tauri/portal-core/src/domains/agents/` or `src/routes/agents/run/` — it's mid-flight on
the same files from a different angle (full P1 review-fix pass vs. this session's live-phone
QA pass); the two overlap on `claude_code.rs` (this session's QA-107 fix) and should be
compatible but haven't been cross-verified together.

## State: **nothing committed this session.** Everything below is uncommitted working-tree changes.

The `agents` domain's own uncommitted baseline (semaphore fix, capability split, QA-106 fix,
etc.) predates this session — see `.claude/handoff.md` for that. This session added fixes on
top of it plus the devices/pairing fixes below. Portal board task **#247** tracks this sweep.

## Goal

Test the paired Android phone (Samsung, `adb-R5GL150LRCK-N75UEH`, connected over wifi adb,
shows in-app as "Tanveer's S26 Ultra") against Portal Desktop's Devices pairing page and the
Agents → Run launcher (coder/terminal backends), file what's broken, fix it, verify it.

## What got done

A `qa-tester` subagent swept Devices + Agents Run against the real running app (CDP `:9222`,
real backend). Found and closed, in order:

| id                                                 | severity | what                                                                                                                                                  | status                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| QA-106                                             | major    | `agents_list_backends` failure was silent, empty Select forever                                                                                       | **verified** (pre-existing fix, confirmed live both normal + forced-error paths)                                                                                                                                                                                                                                                                                                                                                                                                                        |
| QA-107                                             | blocker  | axum route `"/hook/:token"` — old pre-0.7 syntax, panicked at router construction on every `agents_start_run`, hung "Start run" forever with no error | **fixed**, `cargo check`/clippy clean. Partially re-confirmed live (saw a run get past "Starting…" into conversation state) before verification was interrupted — **not yet formally verified**.                                                                                                                                                                                                                                                                                                        |
| QA-108                                             | minor    | `invokeClient.ts`'s `new Error(String(error))` rendered non-Error transport rejections as literal `[object ArrayBuffer]`                              | **fixed** (now uses the existing `getErrorMessage` helper), `pnpm check`/lint clean. **Not yet re-verified live.**                                                                                                                                                                                                                                                                                                                                                                                      |
| QA-109                                             | minor/ux | unchecked shadcn `Switch` track invisible in dark theme (17 places; `--input` token aliased to a border-tuned value)                                  | **fixed** (`src/app.css`, dedicated `0 0% 32%` dark value), confirmed live via contrast measurement before/after. **Not yet re-verified by a fresh qa-tester pass**, but the fixing coder's own before/after numbers are solid evidence.                                                                                                                                                                                                                                                                |
| QA-116 _(filed as QA-110, renumbered — see below)_ | major    | generating a pairing QR counted as "Paired" immediately, before any scan, and never expired from the list even after the 5-min window lapsed          | **fixed**, 2 rounds + reviewer clean on both rounds. Round 1: `devices/+page.svelte` excludes unscanned `until_revoked`/`expires_at`-set rows from "paired". Round 2 (closing a reviewer-found gap): `PairDeviceCard.svelte` polls every 3s for up to 5min while a QR is shown, so a real scan shows up without manual refresh (a clean one-shot backend event wasn't viable — `verify_token` runs in `portal-core`, which has no `AppHandle` to emit from). **Not yet re-verified live by qa-tester.** |

**Filed, not fixed (intentionally, to avoid over-scoping this sweep):**

- QA-117 _(filed as QA-111)_ — minor: unscanned QR grant can't be revoked/cancelled during its live window (now invisible everywhere, not just "paired").
- QA-118 _(filed as QA-112)_ — minor: `pairing_service.rs`'s `verify_token` swallows its first-use DB-update error silently (`let _ = ...update(m)`).
- QA-113 — polish: expired, never-scanned QR rows never get pruned from the DB.
- QA-114 — minor: a single failed devices poll (QA-116's 3s poll) can wipe the whole QR card mid-pairing, because `PageError` unmounts `PairDeviceCard` on any `devicesQ.error` even with stale data present.
- QA-115 — polish: Refresh button flickers busy every ~3s while a QR is on screen (bound to shared `isFetching`, which the poll also flips).

**Open question, not settled:** code-reading only suggested "Accept & remember" in Agents → Run
might be a no-op (frontend drops the remember flag, `agents_decide_permission`'s signature has
no such parameter) — never confirmed live because QA-107 blocked every run from starting.
**Check `.claude/handoff.md` first** — that session's own QA-112 (different QA-112, see
renumbering note below) independently found and already decided this exact thing: hidden the
remember control for the Agents route rather than implement real persistence. If that fix has
landed, this question is already closed; don't re-investigate or re-file it.

## ID collision — already resolved, but know why

This session's `qa-tester` filed new issues as QA-110/111/112, **not knowing** a different
concurrent session's `qa-tester` had _also_ independently filed its own QA-110/111/112 (about
the Agents-run approval banner / end-run / accept-and-remember — a completely different topic,
see `.claude/handoff.md`). The file-based board has no shared counter across concurrent
sessions, so this collided: two different files both claiming the same `id:`. **This session's**
three were renamed and re-numbered to the next free slots to fix it:
`QA-110-unscanned-qr-...` → `QA-116-...`, `QA-111-unscanned-qr-grant-...` → `QA-117-...`,
`QA-112-verify-token-...` → `QA-118-...` (frontmatter `id:` fields and all cross-references
between QA-113/114/115/117/118 updated to match). **If you see a stray reference to "QA-110"
meaning the QR-ghost-device bug in old chat/agent transcripts, it means QA-116.** The other
session's QA-110/111/112 (agents-run topic) are untouched and still theirs.

**If you file more issues from here**, `grep -o 'id: QA-[0-9]*' .claude/qa/issues/*.md | sort -t- -k3 -n | tail` first to see the actual highest id in use — don't trust that the filenames you can see are the only ones in flight; other concurrent sessions may be mid-filing too.

## What's left (in order)

1. **Re-verify QA-107/108/109/116 live.** The qa-tester subagent doing this was stopped mid-task
   (user asked to wrap up and stop running work) — it had gotten as far as confirming a run now
   gets past "Starting…" (QA-107 looks genuinely fixed) before being killed. None of the four got
   a final `verified`/`reopened` write to its issue file this session. Dispatch a fresh
   `qa-tester`: "Verify QA-107, QA-108, QA-109, QA-116" (scope note: QA-116, not QA-110 — see
   collision note above), same checks as originally briefed (QA-107: start a coder run + a
   terminal-tool-call run, exercise approval banner/interrupt/cancel; QA-108: re-run the forced-
   transport-failure repro; QA-109: visual check Devices switches in dark theme; QA-116: generate
   a QR → confirm not "paired", then **have the real phone actually scan it** → confirm it
   appears within ~3s with no manual refresh).
2. **While verifying QA-107, settle the "Accept & remember" question** (see above) — check
   `.claude/handoff.md`'s own QA-112 fix status first; only re-investigate if that fix hasn't
   landed.
3. **Decide on QA-117/118/113/114/115** (all still `open`, none fixed) — triage per
   `.claude/qa/README.md` (blocker/major/minor → fix; polish → fix only if touching those files
   anyway). None are blockers; QA-114 (poll can wipe the QR card) is the one worth prioritizing
   since it undermines QA-116's own fix under a real backend hiccup.
4. **Gates**: `pnpm check` (0 errors) and `pnpm lint` (clean except the pre-existing, unrelated
   `.claude/handoff.md` prettier formatting nit — not this session's file, don't touch it) were
   last run clean as of this handoff, on top of _all_ uncommitted changes in the tree (this
   session's + the concurrent agents-domain session's). `cargo check`/`clippy` were last
   confirmed clean by the QA-107 fix itself (backend-only diff); re-run if more Rust changes land
   before committing.
5. **Commit.** Nothing from either concurrent session is committed. Coordinate with whoever owns
   `.claude/handoff.md`'s work before committing anything under the shared `agents` domain files
   (`claude_code.rs` especially — both sessions touched it). This session's own clean slice
   (`src/app.css`, `PairDeviceCard.svelte`, `devices/+page.svelte`, `invokeClient.ts`, and the
   QA-10[6-9]/11[3-8] issue files) could reasonably be its own commit once verified.

## Files that matter (this session's slice)

- `src-tauri/portal-core/src/domains/agents/claude_code.rs` — QA-107's one-line fix
  (`:token`→`{token}`). **Shared with `.claude/handoff.md`'s session** — this file has their
  in-flight work too (permission-hook UUID, etc.). Check their handoff before further edits here.
- `src/lib/utils/invokeClient.ts` — QA-108 fix, shared helper, low risk, no signature change.
- `src/app.css` — QA-109 fix, dark-theme `--input` token value only.
- `src/routes/devices/+page.svelte`, `src/lib/domains/devices/components/PairDeviceCard.svelte`
  — QA-116 fix (both rounds). Not touched by the other concurrent session (confirmed by their
  own handoff's exclusion list).
- `.claude/qa/issues/QA-106.md` through `QA-109*.md`, `QA-113*.md` through `QA-118*.md` — this
  sweep's issue trail. `QA-106` is `verified`; `107/108/109/116` are `fixed`-pending-reverify;
  `113/114/115/117/118` are `open`, not yet picked up.

## Things that bit this session

- One of the fixing coders (QA-109) ran `pnpm dev` mid-task, which triggered a `predev` hook
  (`dev-instance.mjs --kill-port`) that killed the already-running dev instance's vite server and
  briefly dropped its webview into `chrome-error://` — recovered by navigating the CDP page back,
  no data lost, but **don't run `pnpm dev` against a port another instance already owns**; use
  `node scripts/dev-instance.mjs <n>` for an isolated instance per the project's own convention
  (see `.claude/handoff.md`'s "Things that bit this session" for the same lesson from the other
  side).
- Same coder also `git stash`'d mid-task to isolate a lint failure, which caught **everyone's**
  concurrent uncommitted changes (14 files across both sessions), not just its own — restored
  everything via `git checkout stash@{0} -- <path>` and confirmed byte-identical, but a redundant
  `stash@{0}` entry is still sitting in `git stash list` (confirmed content-identical to the
  current working tree — safe to `git stash drop stash@{0}` whenever, not urgent). **Never `git
stash` in this checkout without `-u` scoped to specific paths** — multiple concurrent sessions
  are active here (confirmed via `ListAgents`: 328 peer sessions, several explicitly on this
  repo) and a bare `git stash` grabs everyone's work.
- This is a heavily multi-session checkout right now. Always `git status --short` before assuming
  a modified/untracked file is yours or is "stray."
