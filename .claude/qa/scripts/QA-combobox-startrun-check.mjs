// Confirms Start run fires correctly using a combobox-selected cwd, and the
// resulting session persists that exact path.
import { chromium } from '@playwright/test';

const cdpPort = process.env.CDP_PORT ?? '9222';
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`, { timeout: 2000 });
const page = browser.contexts()[0].pages()[0];

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/run'));
await page.waitForLoadState('load');
await page.waitForTimeout(1200);

const cwdBefore = await page.getByRole('combobox').nth(1).innerText();
console.log('cwd shown in combobox before start:', cwdBefore);

await page.getByLabel('First message').fill('Say "qa-combobox-startrun-ok" and stop. Do not use any tools.');
await page.getByRole('button', { name: 'Start run' }).click();
await page.waitForTimeout(2500);
const headerText = await page.locator('body').innerText();
console.log('run started:', headerText.includes('Run in progress'));

await page.getByRole('button', { name: 'End run' }).click().catch(() => {});
await page.waitForTimeout(1000);

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/sessions'));
await page.waitForLoadState('load');
await page.waitForTimeout(1000);
const firstRowText = await page.locator('table tbody tr').first().innerText();
console.log('most recent session row:', firstRowText.replace(/\n/g, ' | '));

await browser.close();
