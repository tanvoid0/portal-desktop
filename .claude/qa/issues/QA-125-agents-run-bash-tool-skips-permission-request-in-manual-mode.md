---
id: QA-125
title: "Agents > Run: Bash tool calls execute with no permission request at all, even in manual permission mode"
status: wontfix
severity: major
kind: bug
area: agents/run
mode: tauri
found: 2026-11-11
---

## Use case

Starting a run from Agents > Run launches `claude` with `--permission-mode manual`
(`src-tauri/portal-core/src/domains/agents/claude_code.rs:105-109`, doc comment there says
"manual asks for everything"). The user relies on the "Approval required" banner to review
and gate every tool call, including shell commands, before they run.

## Steps

1. Agents > Run. Pick Claude Code, any cwd, prompt: "Use the Bash tool to run exactly:
   echo QA-full-check. Then stop, do nothing else."
2. Start run. Watch the feed.

## Expected

An "Approval required" banner appears for the Bash tool call before it executes (same as
it does for Write — see repro step 2 of QA-126, which verifies Write does ask).

## Actual

The Bash tool call runs immediately with no approval banner, ever. Reproduced twice,
independently, on two different runs (one on a fresh instance, one on the long-running
QA dev instance), waiting live 90s each time with nothing navigating away in between —
the feed goes straight from "Running Bash" to the command's output, no banner. The
session's own persisted Timeline tab confirms it after the fact: `Tool call: Bash` is
immediately followed by `Tool result`, with no `permission_request`/`permission_resolved`
event in between at all (screenshot). The same run type with a Write tool call *does*
correctly show the banner and persist both permission events (see QA-126's evidence) —
so this isn't the hook relay being broken in general, it's specific to Bash (or at least,
this Bash command) in "manual" mode.

Whether this is Claude Code's own CLI silently fast-pathing commands it judges "safe" even
under `--permission-mode manual`, or a gap in how the hook relay is wired for Bash
specifically, wasn't tracked down — flagging the observed behavior either way, since the
product's own permission promise ("manual asks for everything") does not hold for Bash
today, and a user could reasonably expect shell commands in particular to always be gated.

## Evidence

- `.claude/qa/shots/QA-bash-no-approval-timeline.png` — Timeline tab for a session that ran
  `echo QA-full-check` via Bash: `Tool call: Bash` → `Tool result`, no permission events.
- `.claude/qa/shots/sweep-agents-run-current-state.png` / `explore-run-just-started.png` —
  live run sitting in "Thinking…"/running with no banner.
- Console: none relevant; this is a hook/CLI behavior gap, not a frontend error.

## Repro

Manual — steps above. (No `.mjs` repro checked in; reproduced via ad hoc Playwright scripts
against CDP during this sweep, not preserved as a named QA script.)

## Fix

Traced, not fixed — this is the Claude Code CLI's own behavior, not a gap in
Portal's hook wiring.

Portal's permission-hook relay (`claude_code.rs`'s `spawn_permission_hook`/
`write_hook_settings`) is entirely tool-agnostic: it installs one
`PermissionRequest` http hook with `"matcher": "*"` and forwards whatever
`tool_name`/`tool_input` the CLI itself decides to POST — there is no
Bash-specific code anywhere in `claude_code.rs` that could be swallowing or
skipping a request.

To find out whether the CLI ever calls the hook for Bash at all, I bypassed
Portal entirely and ran the real `claude` CLI (v2.1.317, same version Portal
shells out to) by hand, reproducing Portal's exact invocation: same flags
(`-p --input-format stream-json --output-format stream-json --verbose
--include-partial-messages --permission-mode manual --settings <path>
--setting-sources ""`), same `--settings` shape (one `PermissionRequest` http
hook, `matcher: "*"`), a local mock hook server logging every POST it
receives.

- Prompt: "Use the Write tool to create a file..." → the mock hook *did*
  receive a POST (`tool_name: "Write"`), and the final `result` line's
  `tool_result_meta` read `"permission_decision":{"decision":"accept",
  "source":"hook","reason_type":"hook"}` — hook correctly consulted.
- Prompt: "Use the Bash tool to run exactly: echo QA125trace..." → the mock
  hook received **nothing** for the whole run, and the `result` line's
  `tool_result_meta` read `"permission_decision":{"decision":"accept",
  "source":"config","reason_type":"subcommandResults"}` — the CLI itself
  auto-accepted the Bash call via some internal "known-safe subcommand"
  classification, before the hook relay (or `--permission-mode manual`
  generally) ever got a say.

So: the hook plumbing works identically for every tool it's actually asked
about (proven by Write); Bash specifically never reaches it because the CLI
short-circuits certain commands it judges safe via its own `"source":
"config"`/`"reason_type":"subcommandResults"` path. `claude --help` has no
documented flag to disable this (checked `--permission-mode`'s own choices
and the safe-mode/allow/deny-list flags — none of them touch it), and
`--setting-sources ""` (which Portal already passes to isolate the run from
the user's real settings) doesn't suppress it either, which points to it
being baked into the binary rather than settings-driven. Not fixable from
Portal's side without the CLI exposing a way to turn it off.

No files changed. No other QA ids affected (QA-126's fix is independent).

Checks run: none applicable (investigation only, no code changed in this
repo for this issue).

## Verification

2026-11-11, tauri mode, isolated instance 2 (CDP :9223). Re-read the `## Fix`
investigation — the mock-hook trace (Write hit, `echo QA125trace` via Bash
never hit, `"source":"config"/"reason_type":"subcommandResults"`) is solid,
reproducible evidence that this is the `claude` CLI's own internal
classification, upstream of Portal's hook relay and upstream of
`--permission-mode manual` itself. Nothing in `claude_code.rs` is Bash-aware,
so there's no Portal-side code change that fixes this — confirmed by reading
the file, no tool-specific branching exists anywhere in the hook wiring.

Verdict: **`needs-decision`**, not `wontfix` — this is genuinely unfixable
from Portal's side today (no CLI flag found to disable the fast-path, checked
alongside the fix), but it's a product promise that doesn't hold, not a
no-op: the Run page's own doc comment says "manual asks for everything" and a
user picking "manual" would reasonably expect shell commands specifically to
always be gated, since those are the highest-blast-radius tool call. Owner
call needed: (a) accept the CLI's behavior and soften/correct the "asks for
everything" claim wherever it's shown to the user (currently just a doc
comment, not user-facing copy — but this is also worth surfacing as a caveat
if "manual mode" is advertised to a human anywhere in product copy), or
(b) have Portal pre-empt the CLI's fast path for Bash specifically — e.g.
reject-and-reprompt in Portal's own hook server before forwarding "accept",
or detect `"source":"config"` and surface a banner retroactively from the
tool_call event instead of the (never-fired) permission_request event. Not
touching status beyond `needs-decision`; leaving `wontfix` to the lead/owner
if they agree (a) is the call.

## Owner decision (2026-11-11)

(a): accept the CLI's fast-path behavior as-is, fix the copy. Doc comment in
`claude_code.rs` updated to stop claiming "manual asks for everything" and
note the Bash fast-path + this issue instead. No other user-facing copy made
the same claim. Status → `wontfix`.
