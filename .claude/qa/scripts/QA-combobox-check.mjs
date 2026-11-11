// Checks the Agents > Run working-directory combobox: filter, pick, persist on reload.
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

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/run'));
await page.waitForLoadState('load');
await page.waitForTimeout(1000);

const combobox = page.getByRole('combobox').nth(1);
await combobox.click();
await page.waitForTimeout(400);
await shot(page, 'QA-combobox-open-unfiltered');

await page.getByPlaceholder('Type to filter projects…').fill('portal_desktop');
await page.waitForTimeout(400);
const items = await page.locator('[data-slot="command-item"]').allInnerTexts();
console.log('filtered items for "portal_desktop":', items.length, items.slice(0, 5));
await shot(page, 'QA-combobox-filtered-portal-desktop');

// Pick the first filtered item.
await page.locator('[data-slot="command-item"]').first().click();
await page.waitForTimeout(300);
const valueAfterPick = await combobox.innerText();
console.log('combobox text after pick:', valueAfterPick);

// Reload the route — should remember the pick via localStorage.
await page.reload();
await page.waitForLoadState('load');
await page.waitForTimeout(1200);
const valueAfterReload = await page.getByRole('combobox').nth(1).innerText();
console.log('combobox text after reload:', valueAfterReload);
console.log('persisted correctly:', valueAfterReload === valueAfterPick);
await shot(page, 'QA-combobox-after-reload');

const lsValue = await page.evaluate(() => localStorage.getItem('portal-agents-last-cwd'));
console.log('localStorage portal-agents-last-cwd:', lsValue);

await browser.close();
