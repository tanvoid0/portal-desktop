// Re-verify QA-127 scenario B on the current route (/ai/agents, sidebar +
// detail panel — the old separate /agents/sessions table route is gone).
// Checks whether a session (identified by a snippet of its first-message
// title, shown as the sidebar row's label) shows a terminal status with an
// end time, instead of floating at "running" forever — which is what
// `reconcile_stale_sessions()` (src-tauri/src/lib.rs:292) is supposed to fix
// on the next app startup after a process that left it "running" is gone.
//
//   node .claude/qa/scripts/QA-127-stale-session-check.mjs "<row text snippet>"
import { open } from './app.mjs';

const needle = process.argv[2];
if (!needle) {
  console.error('usage: node QA-127-stale-session-check.mjs "<sidebar row text snippet>"');
  process.exit(1);
}

const app = await open('ai/agents');
const { page } = app;

const row = page.locator('[role="group"]', { hasText: needle }).first();
await row.waitFor({ timeout: 10000 });
await row.click();
await page.waitForLoadState('load');
await page.waitForTimeout(1000);
console.log('screenshot:', await app.shot('QA-127-stale-running-session-detail'));

const bodyText = await page.locator('body').innerText();
console.log('status line contains "Running":', bodyText.includes('Running'));
console.log('status line contains "Ended":', bodyText.includes('Ended'));
console.log('has Transcript tab:', bodyText.includes('Transcript'));
console.log('has Timeline tab:', bodyText.includes('Timeline'));
console.log('has Agents tab (should be false — known pending decision):', bodyText.includes('Agents tab'));

await app.done();
