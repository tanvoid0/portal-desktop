// Starts an agent run and deliberately leaves its approval pending (does NOT
// accept/reject or end it) — for manually killing the app process mid-run and
// confirming `reconcile_stale_sessions()` (src-tauri/src/lib.rs:292) flips it
// to "interrupted" with an end time on the next startup, and that it's still
// listed afterward. Written as a file (not an inline `node -e`) because a
// Windows path's backslashes get mangled into control chars when round-tripped
// through Git Bash's `-e "..."` quoting.
import { open } from './app.mjs';

const SCRATCH =
  'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\af91f09d588336e8\\bfcca801-2c13-40e4-8cde-05136c8ac849\\scratchpad';
const FILENAME = process.argv[2] ?? 'qa127-killtest.txt';

const app = await open(`ai/agents?cwd=${encodeURIComponent(SCRATCH)}`);
const { page } = app;

await page.waitForTimeout(2000); // let backends/projects queries settle before interacting
await page
  .getByLabel('First message')
  .fill(
    `Use the Write tool to create a file named ${FILENAME} with content "kill-test" in the current directory. Then stop.`
  );
const startBtn = page.getByRole('button', { name: 'Start run' });
await startBtn.waitFor({ state: 'visible' });
if (await startBtn.isDisabled()) {
  console.log('Start run is disabled — aborting rather than clicking blind.');
  await app.shot('QA-127-i7-start-disabled');
  await app.done();
  process.exit(1);
}
await startBtn.click();
await page.waitForTimeout(1000);
console.log('url right after click:', page.url());

await page.getByText('Approval required', { exact: false }).first().waitFor({ timeout: 90000 });
const url = page.url();
console.log('PENDING SESSION — leaving it unresolved. URL:', url);
await app.shot('QA-127-i7-pending-before-kill');
await app.done();
