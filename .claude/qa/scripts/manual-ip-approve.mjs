import { chromium } from 'playwright';

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const contexts = browser.contexts();
for (const ctx of contexts) {
  for (const p of ctx.pages()) {
    console.log('page:', p.url());
  }
}
const page = contexts[0].pages().find((p) => p.url().includes('/devices')) ?? contexts[0].pages()[0];
if (!page) throw new Error('no /devices page found');

await page.bringToFront();
await page.waitForTimeout(500);

const bodyText = await page.innerText('body');
console.log('--- current page text (truncated) ---');
console.log(bodyText.slice(0, 2500));

const card = page.locator('text=Tanveer\'s S26 Ultra').locator('xpath=ancestor::*[self::li or self::div][.//button[contains(., "Allow")]]').first();
await card.screenshot({ path: '.claude/qa/shots/manual-ip-pending-card.png' }).catch(async (e) => {
  console.log('card screenshot failed, falling back to full page:', e.message);
  await page.screenshot({ path: '.claude/qa/shots/manual-ip-pending-card.png', fullPage: true });
});

await browser.close();
