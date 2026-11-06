---
name: qa-loop
description: Run the test → fix → verify loop on Portal Desktop between the qa-tester and coder agents, with the QA board (.claude/qa/issues/) as their channel. Use when the user says "QA this", "test X and fix what you find", "run the qa loop", "sweep the projects page", or after a feature lands and needs testing. Args — a scope (route, domain, feature, "changes" for the current diff) or QA ids to finish.
---

# QA loop

You are the lead. Subagents cannot spawn each other, so you carry the work between them;
the board carries the detail. Contract: `.claude/qa/README.md`. Read it once.

**Cap: never more than 10 agents at once** (user's global rule). Never two coders on the same files.

## 1. Scope

- Route / domain / feature → that.
- `changes` → routes behind `git diff --name-only main...` and the working tree.
- QA ids → skip to step 4 with those.
- Nothing given → ask which area.

Check the board for issues already open in scope: `grep -l "^status: \(open\|reopened\|fixed\)" .claude/qa/issues/*.md`.

## 2. Is the real app reachable?

`netstat -ano | grep -E ":(1420|9222) .*LISTEN"`. If the scope has data flows and only `:1420`
answers (app running without CDP), ask the owner once: restart it with
`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 pnpm tauri:dev`, or go on
in browser mode (UI only). Don't restart their app yourself.

## 3. Sweep

Spawn `qa-tester`: "Sweep <scope>. Mode/ports: <what step 2 found>." Keep its agent id —
you will send it the verify round so it keeps its context.

## 4. Triage

Read the new issue files (not just the summary).

- `needs-decision` → batch into one AskUserQuestion to the owner; apply the answers to the issue (fix → `open`, not a bug → `wontfix`).
- `blocker` / `major` / `minor` → fix.
- `polish` → fix when it is in files already being touched, else leave open and list it.

## 5. Fix

Group issues by the files they will touch. One `coder` per group: "Fix QA-003, QA-007 per
`.claude/qa/README.md`." Groups sharing files run one after another. Keep each coder's id.

If a fix has real logic (a branch, a loop, a data path, 3+ files), run `reviewer` on that diff;
send its findings back to the same coder with SendMessage.

## 6. Verify

SendMessage the same `qa-tester`: "Verify QA-003, QA-007." It sets `verified` or `reopened`.

## 7. Repeat or stop

Reopened → back to step 5, same coder via SendMessage. Stop after 3 rounds. An issue reopened
twice goes to the owner with both sides' notes — don't let the agents ping-pong.

## 8. Gates and report

Run inline: `pnpm check && pnpm lint`, plus `cargo check` from `src-tauri/` if Rust changed.
Report to the owner: verified · still open (and why) · needs-decision · not testable in the
mode used. No commit unless they ask.
