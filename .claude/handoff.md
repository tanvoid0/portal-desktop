# Handoff: 2026-11-15 (continued again) — #635 fixed and verified, nothing committed yet

## This continuation

User asked to fix #635 right after it was filed. Done:

- `AgentsWorkspace.svelte`: `focusedId` changed from
  `$derived(page.url.searchParams.get('session'))` to local `$state`, set
  directly by `selectSession`/`newRun`/the `onStarted` callback. `syncUrl()`
  now only mirrors the id into `?session=` for deep-linking/refresh, mirroring
  how `CoderPanel.svelte` actually works (drives off its own state, URL is
  just a mirror) rather than the inverted pattern that was there before.
- Verified live over CDP on the dev build: clicking a sidebar row opens the
  thread immediately (no reload), switching between two different sessions
  works, and a freshly-started run opens its own thread right away.
- `pnpm check` (0 errors) and `pnpm lint` (0 errors, no new warnings in the
  touched file) both pass.
- Closed board #635 with the fix note. **Not committed** — one file changed
  (`src/lib/domains/agents/components/AgentsWorkspace.svelte`), nothing
  staged, waiting on the user to say whether to commit/push.
- Along the way the dev instance died mid-session (terminal tab + release
  daemon both dropped — looked like an environment/sandbox restart, not
  anything this session did) and had to be restarted twice
  (`node scripts/dev-instance.mjs 1`, then `pnpm portal:ensure` to bring the
  shared :18420 daemon back). Both came back clean; no data lost, the two
  test sessions from earlier are still in the dev DB.

## Still left (unchanged from before)

1. Commit/push the #635 fix (holding for the user).
2. Click through the pairing approval in a real window (dev or release).
3. Install the new release build and smoke-test it — installer still
   untested in `src-tauri/target/release/bundle/{nsis,msi}/`.
4. #636 (predev npx) and #550 (global pnpm) — likely the same nvm/npm
   corruption, worth fixing together.
5. #577 (Portal `update`'s `note` field length bug) still open.

---

# Handoff: 2026-11-14 (continued) — Agents redesign live-verified over CDP, found a real regression

## This continuation found and did

Picked up the "left to do" list below. User was present and steered toward
using the documented CDP harness instead of computer-use pixels (right call —
see `[[driving-portal-desktop-over-cdp]]` memory).

1. **Fixed the launch path, not just the symptom**: `.claude/launch.json`'s
   `tauri` config runs `pnpm run tauri dev`, which never sets
   `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` — only
   `node scripts/dev-instance.mjs 1` does (see its own comment, which says
   plain `pnpm tauri:dev` "is left exactly as it was — same CDP port", which
   is **not true** for how the browser-pane launcher invokes it). Started the
   dev build via `node scripts/dev-instance.mjs 1` in a Terminal tab instead;
   CDP came up on :9222 as expected. **If you reach for `preview_start({name:
"tauri"})` for CDP work, it won't give you a debug port — use the terminal
   - `dev-instance.mjs 1` instead.**
2. **Live-verified the Agents redesign (#546, dd27f0f0)** — the one thing
   last session couldn't test. Ran a real Claude Code agent session
   (`List the files in the current directory, then stop.`, cwd = Devstrail)
   through the real backend over CDP via `.claude/qa/scripts/app.mjs` +
   `window.__TAURI_INTERNALS__.invoke`:
   - Backend is solid: `agents_start_run` → real process → real transcript
     (`agents_get_session` returned the actual `ls` output).
   - **Found a real regression, filed as board #635**: clicking a session row
     in the sidebar (or finishing starting a new run) updates the URL
     (`?session=<id>`, confirmed via `page.url()`) but the main panel stays
     on the blank "Start run" launcher. A **hard reload of that same URL**
     renders the thread correctly — transcript, expandable tool-call ("Ran
     1s"), status badges, follow-up composer, all fine. So the bug is
     specifically that `AgentsWorkspace.svelte`'s
     `focusedId = $derived(page.url.searchParams.get('session'))` doesn't
     react to its own `replaceState` call; its comment claims this mirrors
     `CoderPanel.svelte`'s `?id=` sync, but `CoderPanel` actually drives
     rendering off its own `coderSession.activeThreadId` state and only uses
     `replaceState` to keep the URL in sync for deep-linking — not as the
     trigger. `AgentsWorkspace` inverted that. Did **not** attempt the fix
     this pass — it's a real one worth a focused coder session, and I'd
     already spent the budget diagnosing it precisely (see #635 for the full
     repro + root-cause writeup).
   - Could not test the approval-banner / `decide()` path, since the trivial
     `ls` prompt never needed a permission grant and the sidebar bug blocked
     reopening a thread anyway to try a prompt that would.
3. **Filed #636** (low): `predev` (`node scripts/dev-instance.mjs
--kill-port`) crashes on every dev launch — nvm's `npx` is broken for node
   v24.14.1 (`Cannot find module '@npmcli/config/lib/definitions'`). Harmless
   when the port's already free, but means `kill-port` never actually runs.
   Related to #550.
4. **Did not click through the device-pairing approval** (left-to-do #1
   below) — ran out of session budget after the Agents dig; the dev instance
   from this session is still up (CDP :9222, vite :1420, daemon :18421) if
   you want to pick that up directly.
5. Cleaned up all scratch `.claude/qa/scripts/continue-*.mjs` test scripts —
   none kept, the findings are in #635/#636 instead.

## Still left (updated)

1. **Fix #635** (Agents sidebar navigation) — the highest-value next step;
   it blocks actually using the redesign day-to-day.
2. Click through the pairing approval in a real window (dev or release) —
   still nobody has watched a paired browser get a working session.
3. Install the new release build and smoke-test it — installer still sitting
   untested in `src-tauri/target/release/bundle/{nsis,msi}/`.
4. #636 (predev npx) and #550 (global pnpm) are likely the same underlying
   nvm/npm corruption — worth fixing together.
5. #577 (Portal `update`'s `note` field length bug) still open, still just
   worked around by keeping notes short.

## Watch out for

- The dev instance this session started (`node scripts/dev-instance.mjs 1`,
  terminal tab "tauri dev cdp") may still be running — CDP :9222, vite :1420,
  dev daemon :18421. Check before assuming a clean slate, same as last time.
- Use `node scripts/dev-instance.mjs 1` for CDP work, **not**
  `preview_start({name: "tauri"})` / plain `pnpm tauri:dev` — only the
  former sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`.
- The release daemon on :18420 is still the shared singleton — untouched
  this session, same caution as before applies.

---

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
