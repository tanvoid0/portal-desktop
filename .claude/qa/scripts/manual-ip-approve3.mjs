import { chromium } from 'playwright';

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages().find((p) => p.url().includes('/devices'));
await page.bringToFront();

const grid = page.locator(
  "xpath=//h3[text()='Pending requests']/following-sibling::div[contains(@class,'grid')][1]"
);
const cards = grid.locator(':scope > div');
const cardCount = await cards.count();
console.log('pending cards:', cardCount);

let target = null;
for (let i = 0; i < cardCount; i++) {
  const c = cards.nth(i);
  const text = await c.innerText();
  if (text.includes("Tanveer's S26 Ultra")) {
    target = c;
    break;
  }
}
if (!target) throw new Error('could not find the S26 Ultra pending card');

for (const cap of ['coder', 'terminal', 'agents', 'dashboard', 'projects', 'tasks']) {
  const label = target.locator('label').filter({ hasText: new RegExp(`^${cap.replace('.', '\\.')}$`) });
  const btn = label.locator('button');
  await btn.click();
  console.log('toggled', cap);
}

// Grant "Until revoked" so the token stays usable for the follow-up control test.
await target.getByText('Until revoked', { exact: true }).click();

await target.screenshot({ path: '.claude/qa/shots/manual-ip-card-before-allow.png' });

await target.getByRole('button', { name: 'Allow' }).click();
await page.waitForTimeout(1000);

await page.screenshot({ path: '.claude/qa/shots/manual-ip-after-allow.png', fullPage: true });

await browser.close();
