import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);

async function revokeOneUnnamed() {
  const clicked = await app.page.evaluate(() => {
    const cards = Array.from(document.querySelectorAll('[data-slot="card"]'));
    for (const card of cards) {
      if (card.textContent?.includes('Unnamed device')) {
        const btn = Array.from(card.querySelectorAll('button')).find(
          (b) => b.textContent?.trim() === 'Revoke'
        );
        if (btn) { btn.click(); return true; }
      }
    }
    return false;
  });
  return clicked;
}

let revoked = 0;
for (let i = 0; i < 5; i++) {
  const ok = await revokeOneUnnamed();
  if (!ok) break;
  await app.page.waitForTimeout(400);
  const confirmBtn = app.page.locator('button').filter({ hasText: /^Revoke device$/ });
  if ((await confirmBtn.count()) > 0) {
    await confirmBtn.last().click();
    await app.page.waitForTimeout(700);
    revoked++;
  } else {
    break;
  }
}
console.log('revoked count:', revoked);
const text = await app.page.evaluate(() => document.body.innerText);
console.log('paired count now:', text.match(/Paired\s+(\d+)/)?.[1]);
console.log('still mentions Unnamed device:', text.includes('Unnamed device'));
await app.shot('QA-110-cleanup2-after');
await app.done();
