// Spot-check: Tab to a session row, Enter/Space opens it.
import { chromium } from '@playwright/test';

const cdpPort = process.env.CDP_PORT ?? '9222';
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`, { timeout: 2000 });
const page = browser.contexts()[0].pages()[0];

await page.goto(page.url().replace(/\/agents\/.*/, '/agents/sessions'));
await page.waitForLoadState('load');
await page.waitForTimeout(1000);

// Tab from the Refresh button toward the first row.
await page.getByRole('button', { name: 'Refresh', exact: true }).focus();
let found = false;
for (let i = 0; i < 5; i++) {
  await page.keyboard.press('Tab');
  const active = await page.evaluate(() => {
    const el = document.activeElement;
    return { role: el?.getAttribute('role'), tag: el?.tagName, text: el?.textContent?.slice(0, 40) };
  });
  console.log('focused:', JSON.stringify(active));
  if (active.role === 'button' && active.tag === 'TR') {
    found = true;
    break;
  }
}
console.log('reached a row via Tab:', found);
if (found) {
  await page.keyboard.press('Enter');
  await page.waitForLoadState('load');
  await page.waitForTimeout(800);
  console.log('url after Enter:', page.url());
}

await browser.close();
