# MCP Important-Only Summaries (cursor-ide-browser)

Goal: reduce credit usage and log noise by extracting only what matters when debugging with MCP browser tools.

When the agent uses `cursor-ide-browser` MCP tools (especially `browser_console_messages` and `browser_network_requests`):

1. Prefer “signal-first” summaries
   - Do not paste raw/full tool outputs into the chat.
   - Summarize the minimum set of facts needed to diagnose and fix the issue.

2. Console messages (`browser_console_messages`)
   - Include only entries with severity `error` or `warn`.
   - Drop `info` and other low-signal messages unless the user explicitly asks for all logs.
   - Dedupe by message text (and, when available, by error name). Show each unique message once plus a count if it repeats.
   - If the console output is large, only include the top N (default 5) by occurrence and/or recency.

3. Network requests (`browser_network_requests`)
   - Include only requests that are failing/high-signal:
     - HTTP status `>= 400`, or
     - requests marked as failed (timeouts, DNS, aborted, etc.), or
     - responses that clearly indicate auth/permission problems (e.g., 401/403).
   - For each included request, show only: `method`, `url` (or pathname), `status` (or failure reason), and the shortest useful identifier (e.g., request id / error code if present).
   - Cap the list to default N=10 to avoid overwhelming the model.

4. UI state (`browser_snapshot`)
   - Use `compact: true` and (when possible) scope with `selector` to the smallest relevant subtree.
   - Summarize only the elements that relate to the user’s reported symptom (missing button, wrong text, disabled control, etc.).

5. Waiting / retry control (`browser_wait_for`)
   - Prefer waiting for specific text conditions over repeated broad polling.

6. Profiling / heavy tools
   - Avoid `browser_profile_start` / `browser_profile_stop` unless the user asks about performance or you have a specific slow-JS hypothesis.

7. End with an action
   - Conclude with the single most likely next step to verify/fix the issue (e.g., “check X request payload” or “confirm Y element renders”).

