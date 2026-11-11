// Cleanup: revoke the "Unnamed device" ghost entry created by QA-110's repro
// (an unscanned QR generated during this sweep).
import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);

const card = app.page.locator('div').filter({ hasText: /^Unnamed device/ }).last();
console.log('found card:', await card.count());

const revokeBtn = app.page.locator('button').filter({ hasText: 'Revoke' });
console.log('revoke buttons total:', await revokeBtn.count());

// Find the Revoke button inside the card that contains "Unnamed device".
const target = app.page.locator('text=Unnamed device').locator('xpath=ancestor::*[self::div][3]');
const targetRevoke = target.locator('button').filter({ hasText: 'Revoke' });
console.log('target revoke count:', await targetRevoke.count());
if (await targetRevoke.count() > 0) {
  await targetRevoke.first().click();
  await app.page.waitForTimeout(500);
  // Confirm dialog (AlertDialog) — click the destructive confirm button.
  const confirmBtn = app.page.locator('button').filter({ hasText: /^Revoke device$|^Revoke$|^Confirm$/ });
  console.log('confirm dialog buttons:', await confirmBtn.count());
  await app.shot('cleanup-unnamed-device-confirm-dialog');
  if (await confirmBtn.count() > 0) {
    await confirmBtn.last().click();
    await app.page.waitForTimeout(800);
  }
}
await app.shot('cleanup-unnamed-device-after');
const text = await app.page.evaluate(() => document.body.innerText);
console.log('still mentions Unnamed device:', text.includes('Unnamed device'));
await app.done();
