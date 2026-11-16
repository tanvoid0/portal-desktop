// Re-verify QA-127 scenario A on the current route: reloading /ai/coder/claude-code
// mid-approval reattaches to the live session (via ?session=<id>, synced by
// AgentsWorkspace.svelte's syncUrl + agentRuns.attach()'s persisted+ring
// backfill) and shows the approval banner again, instead of a blank launcher.
// Updated from the old /agents/run route — same assertions as the original.
import { open } from './app.mjs';

// cwd seeded via the launcher's own `?cwd=` query param (RunLauncherForm's
// `initialCwd`) rather than picking a real tracked project — keeps this a
// QA-owned fixture dir, not the owner's project files.
const SCRATCH =
  'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\af91f09d588336e8\\bfcca801-2c13-40e4-8cde-05136c8ac849\\scratchpad';

const app = await open(`ai/coder/claude-code?cwd=${encodeURIComponent(SCRATCH)}`);
const { page } = app;

await page
  .getByLabel('First message')
  .fill(
    'Use the Write tool to create a file named qa127-reload-verify.txt with content "hello" in the current directory. Then stop.'
  );
await page.getByRole('button', { name: 'Start run' }).click();

console.log('run started, waiting for approval banner...');
await page
  .getByText('Approval required', { exact: false })
  .first()
  .waitFor({ timeout: 60000 });
await app.shot('QA-127-recheck-before-reload-banner-up');
const urlWithSession = page.url();
console.log('banner up, url now:', urlWithSession, '— reloading...');

await page.goto(urlWithSession);
await page.waitForLoadState('load');
await page.waitForTimeout(1500);
await app.shot('QA-127-recheck-after-reload');

const text = await page.locator('body').innerText();
console.log(
  text.includes('Approval required')
    ? 'AFTER RELOAD: banner reappeared (FIX WORKS)'
    : 'AFTER RELOAD: banner missing (still broken)'
);
console.log(
  text.includes('qa127-reload-verify.txt') ? 'path FOUND in reattached banner' : 'path NOT found'
);

// Decide every pending approval so the run doesn't sit blocked after this
// script exits (more than one can be pending at once — extraPending in
// AgentSessionPanel.svelte).
for (let i = 0; i < 5; i++) {
  const acceptBtn = page.getByRole('button', { name: 'Accept once' }).first();
  if (!(await acceptBtn.isVisible().catch(() => false))) break;
  await acceptBtn.click();
  await page.waitForTimeout(1500);
}
await page
  .getByRole('button', { name: 'End run' })
  .click()
  .catch(() => {});

await app.done();
