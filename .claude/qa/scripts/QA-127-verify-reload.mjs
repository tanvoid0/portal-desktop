// Verify QA-127 scenario A: reloading /agents/run mid-approval reattaches to
// the live run and shows the banner again, instead of a blank launcher form.
import { open } from './app.mjs';

const app = await open('agents/run');
const { page } = app;

await page
  .getByLabel('First message')
  .fill(
    'Use the Write tool to create a file named qa127-reload-test.txt with content "hello" in the current directory. Then stop.'
  );
await page.getByRole('button', { name: 'Start run' }).click();

console.log('run started, waiting for approval banner...');
await page.getByText('Approval required', { exact: false }).waitFor({ timeout: 60000 });
await app.shot('QA-127-before-reload-banner-up');
console.log('banner up, reloading /agents/run...');

await page.goto(page.url());
await page.waitForLoadState('load');
await page.waitForTimeout(1500);
await app.shot('QA-127-after-reload');

const text = await page.locator('body').innerText();
console.log(
  text.includes('Approval required')
    ? 'AFTER RELOAD: banner reappeared (FIX WORKS)'
    : 'AFTER RELOAD: banner missing (still broken)'
);
console.log(text.includes('qa127-reload-test.txt') ? 'path FOUND in reattached banner' : 'path NOT found');

// Decide so the run doesn't sit blocked after this script exits.
const acceptBtn = page.getByRole('button', { name: 'Accept once' });
if (await acceptBtn.isVisible().catch(() => false)) {
  await acceptBtn.click();
  await page.waitForTimeout(1500);
}
await page.getByRole('button', { name: 'End run' }).click().catch(() => {});

await app.done();
