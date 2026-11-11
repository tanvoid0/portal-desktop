import { chromium } from 'playwright';
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages()[0];
await page.goto('http://localhost:1421/devices');
await page.waitForTimeout(1500);
await page.screenshot({
  path: 'D:/projects/production/portal/portal_desktop/.claude/qa/screenshots/QA-121-verify-baseline-devices.png',
  fullPage: true,
});
const text = await page.locator('main').innerText();
console.log(text);
await browser.close();
