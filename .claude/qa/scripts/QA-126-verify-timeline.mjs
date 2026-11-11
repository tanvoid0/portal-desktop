// Continuation: accept the pending Write, end the run, open Sessions, open the
// session, check the Timeline tab's "Permission requested: Write" row for detail.
// Connects to the already-open page without navigating away from the pending banner.
import { chromium } from '@playwright/test';
import { existsSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SHOTS = fileURLToPath(new URL('../shots/', import.meta.url));
async function shot(page, name) {
  mkdirSync(SHOTS, { recursive: true });
  let path = `${SHOTS}${name}.png`;
  for (let i = 2; existsSync(path); i++) path = `${SHOTS}${name}-${i}.png`;
  await page.screenshot({ path, fullPage: true });
  return path;
}

const cdpPort = process.env.CDP_PORT ?? '9222';
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`, { timeout: 2000 });
const page = browser.contexts()[0].pages()[0];

await page.getByRole('button', { name: 'Accept once' }).click();
await page.waitForTimeout(3000);
await page.getByRole('button', { name: 'End run' }).click();
await page.waitForTimeout(1500);

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/sessions'));
await page.waitForLoadState('load');
await page.waitForTimeout(1000);

const firstRow = page.locator('table tbody tr').first();
await firstRow.click();
await page.waitForLoadState('load');
await page.waitForTimeout(1000);

const timelineTab = page.getByRole('tab', { name: 'Timeline' });
if (await timelineTab.count()) {
  await timelineTab.click();
  await page.waitForTimeout(500);
}
console.log('screenshot:', await shot(page, 'QA-126-verify-timeline'));

const bodyText = await page.locator('body').innerText();
console.log(
  bodyText.includes('qa126-write-test.txt')
    ? 'TIMELINE: file path FOUND'
    : 'TIMELINE: file path NOT FOUND'
);

await browser.close();
