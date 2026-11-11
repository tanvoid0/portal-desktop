// Checks whether the stuck "running" session (started 16:56, before this
// instance's own 18:03 start) also shows "running" on its own detail page,
// and whether opening it triggers any reconciliation.
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

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/sessions'));
await page.waitForLoadState('load');
await page.waitForTimeout(1000);

// Row with started "Today, 16:56" per earlier screenshot.
const row = page.locator('table tbody tr', { hasText: '16:56' });
await row.click();
await page.waitForLoadState('load');
await page.waitForTimeout(1000);
console.log('screenshot:', await shot(page, 'QA-127-stale-running-session-detail'));

const bodyText = await page.locator('body').innerText();
console.log('contains "running":', bodyText.toLowerCase().includes('running'));

await browser.close();
