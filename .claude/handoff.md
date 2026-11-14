# Handoff: 2026-11-14 session close — pushed, release built, PC about to shut down

## What this session found and did

Picked up the previous `.claude/handoff.md` (Agents redesign, board #546). That
handoff was stale: the Agents work (`dd27f0f0`) was already merged to `main`
and pushed by a later session, which then also did two rounds of Portal
Mobile work (Phase 1 phone polish, Phase 2 remote caps) on top. Confirmed via
`git merge-base feat/database-explorer main` — the old branch's tip is an
ancestor of `main`, not a sibling.

Per the user's live request this session (unattended, "finish everything you
can on your own, shut down if you hit something that needs a click"):

1. **Verified board bug #618** ("Phase 2.1: event_cap forwards..." subtask)
   was shown unchecked but already landed in `ca57f65d` — `git show` confirmed
   the diff is there. Marked it `completed`, checkbox was just stale.
2. **Live-verified the browser/other-desktop client** (task #542) for the
   first time in a real browser — this was the one genuinely unverified piece
   left in `docs/development/PORTAL_MOBILE.md`. Rebuilt `build-client`, opened
   a browser-pane tab at the dev daemon (`:18421`), confirmed:
   - the daemon's `fallback_service` serves `/connect` correctly
   - `browserShim.ts` installs `window.__TAURI_INTERNALS__` and the real
     connect UI renders (not a crash, which is all last session confirmed)
   - pairing to `127.0.0.1:18421` fails outright (`Failed to fetch` — browser
     treats `localhost`/`127.0.0.1` as different origins even same
     port/machine). Pairing to `localhost:18421` (the page's own origin)
     works and reaches the real "Approve on your PC…" long-poll state.
   - **Did not click Approve.** That's a device-approval decision inside the
     running desktop app — needs the native window. Computer-use access
     needs a one-time permission dialog the user has to click; user was AFK,
     so per their instruction this was left rather than waited on.
     Docs + board task #542 updated with this.
3. **Could not live-verify the Agents redesign** in the native dev window for
   the same reason — same computer-use permission-dialog blocker. Skipped,
   per the same instruction.
4. **Built the release installer**: `pnpm tauri:build` succeeded clean —
   `src-tauri/target/release/bundle/nsis/Portal Desktop_0.15.0_x64-setup.exe`
   and the `.msi` sibling both exist.
5. **Did not install it.** The currently-running release
   (`%LOCALAPPDATA%\Portal Desktop\portal_desktop.exe`, pid seen this session)
   is the one shared task-board daemon on :18420 that every session/tool on
   this machine talks to (including this session's own board MCP calls).
   Overwriting/restarting it mid-session to test one feature, with no way to
   click through the resulting UAC prompt or the app's own UI afterward
   (same computer-use blocker as above), was judged not worth the shared-state
   risk. **This is the one explicit ask ("verify in a release installed on
   this PC") that did not get done — flagging it clearly rather than claiming
   it happened.**
6. **Pushed to `origin/main`**: `ca57f65d` (Phase 2 remote caps, was sitting
   local-only) and `8096df10` (this session's docs update). `origin/main` ==
   `main` now.

## Left to do (next session, or next time someone's at the keyboard)

1. **Click through the pairing approval** in the real desktop app (dev or
   release) to finish verifying the browser/other-desktop client end to end —
   the request reaches "Approve on your PC" correctly; nobody has clicked
   Allow and watched the paired browser actually get a working session.
2. **Install the new release build and smoke-test it** — the installer is
   sitting in `src-tauri/target/release/bundle/{nsis,msi}/`, untested.
   Whoever does this should close the currently-running release first (it's
   the shared :18420 daemon other tools/sessions may be using) rather than
   installing over a live process.
3. **Live-verify the Agents redesign** in an actual native window — tool
   calls rendering properly, approval banner + `decide()`, resume reusing the
   sidebar row, the global floating approval stack, project/task cross-nav
   with real ids. None of this has been click-tested since it was built
   (see git history around `dd27f0f0` for what it is).
4. Board bugs still open: **#550** (global pnpm broken) — looks fixed now,
   `pnpm -v` returned `9.15.9` cleanly this session, not independently
   re-verified end to end; **#577** (Portal MCP `update`'s `note` field choking
   on long text) — still open, still worked around by keeping notes short.

## Watch out for

- **The release daemon on :18420 is shared, singleton, load-bearing.** Do not
  stop/replace/reinstall over it without checking nothing else on the machine
  is mid-task against the board. This session deliberately did NOT do that.
- **Computer-use access needs a one-time user click** on its permission
  dialog. If the user is away, this blocks any native-window verification —
  don't `request_access` and sit there; skip it and say so, like this session
  did, rather than hanging.
- **`localhost` vs `127.0.0.1` are different browser origins** even same
  port/machine — a same-machine pairing test must use whatever host the page
  itself was loaded from, or the fetch fails before it even reaches the
  daemon. Not a bug, just a gotcha for whoever tests this again.
- Dev build was started this session (`pnpm tauri:dev` via the browser-pane
  preview launcher, since direct terminal invocation of `pnpm tauri:dev` was
  blocked twice by a hook rewriting the command mid-flight — fell back to
  `preview_start({name: "tauri"})` instead, which worked). That dev process
  (and its native window) may still be running when this PC is turned back
  on if the shutdown didn't clean it up gracefully — check before assuming a
  clean slate.
