// Revoke every "Unnamed device" QR-pairing fixture this verification created.
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

let revokedCount = 0;
for (let i = 0; i < 10; i++) {
  const revokeBtns = app.page.locator('button').filter({ hasText: 'Revoke' });
  const n = await revokeBtns.count();
  if (n === 0) break;
  // Only revoke cards that say "Unnamed device" -- find one and click its Revoke.
  const card = app.page.locator('text=Unnamed device').first();
  if (!(await card.isVisible().catch(() => false))) break;
  const cardRoot = card.locator('xpath=ancestor::*[.//button[contains(., "Revoke")]][1]');
  const btn = cardRoot.locator('button').filter({ hasText: 'Revoke' }).first();
  await btn.click();
  await app.page.waitForTimeout(300);
  // Confirm dialog if present.
  const confirmBtn = app.page
    .locator('button')
    .filter({ hasText: /^(Revoke|Confirm|Yes)$/ })
    .last();
  if (await confirmBtn.isVisible().catch(() => false)) {
    await confirmBtn.click();
  }
  await app.page.waitForTimeout(500);
  revokedCount++;
}
console.log('revoked count:', revokedCount);
await app.page.waitForTimeout(500);
const bodyText = await app.page.evaluate(() => document.body.innerText);
console.log('Unnamed device still present:', /Unnamed device/i.test(bodyText));
await app.shot('QA-116-cleanup-after');
await app.done();
