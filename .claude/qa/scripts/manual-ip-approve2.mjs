import { chromium } from 'playwright';

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages().find((p) => p.url().includes('/devices'));
await page.bringToFront();

// Scope to the pending-request card for Tanveer's S26 Ultra specifically.
const heading = page.getByText("Tanveer's S26 Ultra", { exact: true });
const card = heading.locator('xpath=ancestor::div[.//button[contains(., "Allow")]][1]');

const count = await card.count();
console.log('card matches:', count);

for (const cap of ['coder', 'terminal', 'agents']) {
  const toggle = card.getByText(cap, { exact: true }).locator('xpath=preceding-sibling::button[1] | ./preceding::button[1]');
  console.log(cap, 'toggle count:', await toggle.count());
}

await card.screenshot({ path: '.claude/qa/shots/manual-ip-card-scoped.png' });

await browser.close();
