import { chromium } from 'playwright';

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages().find((p) => p.url().includes('/devices'));
await page.bringToFront();

const grid = page.locator(
  "xpath=//h3[text()='Pending requests']/following-sibling::div[contains(@class,'grid')][1]"
);
const card = grid.locator(':scope > div').first();
console.log(await card.innerHTML());

await browser.close();
