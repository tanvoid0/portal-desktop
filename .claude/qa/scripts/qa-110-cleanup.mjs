// Cleanup: revoke the two "Unnamed device" test fixtures created while
// verifying QA-110 (tasks + coder capability grants claimed via adb curl).
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);

let revoked = 0;
for (let i = 0; i < 5; i++) {
  const unnamed = app.page.locator('text=Unnamed device').first();
  if ((await unnamed.count()) === 0) break;
  const card = unnamed.locator('xpath=ancestor::*[self::div][3]');
  const revokeBtn = card.locator('button').filter({ hasText: 'Revoke' });
  if ((await revokeBtn.count()) === 0) break;
  await revokeBtn.first().click();
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
await app.shot('QA-110-cleanup-after');
await app.done();
