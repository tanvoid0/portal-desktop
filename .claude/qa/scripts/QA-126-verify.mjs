// Verify QA-126 fix end-to-end: Write tool approval banner shows file path
// live, and the Timeline tab's "Permission requested: Write" row shows it too.
import { open } from './app.mjs';

const app = await open('agents/run');
const { page } = app;

await page
  .getByLabel('First message')
  .fill(
    'Use the Write tool to create a file named qa126-write-test2.txt with content "hello" in the current directory. Then stop.'
  );
await page.getByRole('button', { name: 'Start run' }).click();

console.log('run started, waiting for approval banner...');
await page.getByText('Approval required', { exact: false }).waitFor({ timeout: 60000 });
await app.shot('QA-126-verify-live-banner-2');
const liveText = await page.locator('body').innerText();
console.log(
  liveText.includes('qa126-write-test2.txt') ? 'LIVE BANNER: path FOUND' : 'LIVE BANNER: path NOT FOUND'
);

await page.getByRole('button', { name: 'Accept once' }).click();
await page.waitForTimeout(3000);
await page.getByRole('button', { name: 'End run' }).click().catch(() => {});
await page.waitForTimeout(1200);

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/sessions'));
await page.waitForLoadState('load');
await page.waitForTimeout(1000);
await page.locator('table tbody tr').first().click();
await page.waitForLoadState('load');
await page.waitForTimeout(800);

const timelineTab = page.getByRole('tab', { name: 'Timeline' });
await timelineTab.click();
await page.waitForTimeout(500);
await app.shot('QA-126-verify-timeline-2');

const timelineText = await page.locator('body').innerText();
console.log(
  timelineText.includes('qa126-write-test2.txt')
    ? 'TIMELINE: path FOUND'
    : 'TIMELINE: path NOT FOUND'
);

await app.done();
