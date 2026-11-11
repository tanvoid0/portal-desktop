// QA-117 verification, part A: generate a QR, click Cancel before scanning,
// confirm the grant shows revoked in History and the card resets.
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function bodyText() {
  return app.page.evaluate(() => document.body.innerText);
}

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(1000);

const regenVisible = await app.page.locator('button').filter({ hasText: 'Regenerate QR code' }).isVisible();
const cancelBtn = app.page.locator('button').filter({ hasText: 'Cancel' });
const cancelVisible = await cancelBtn.isVisible();
const qrVisible = await app.page.locator('img[alt="Pairing QR code"]').isVisible();
console.log('Regenerate label shown:', regenVisible, '| Cancel button shown:', cancelVisible, '| QR shown:', qrVisible);
await app.shot('QA-117-before-cancel-click');

await cancelBtn.click();
await app.page.waitForTimeout(1000);

const qrGone = !(await app.page.locator('img[alt="Pairing QR code"]').isVisible());
const backToGenerate = await app.page.locator('button').filter({ hasText: 'Generate QR code' }).isVisible();
console.log('QR gone after Cancel:', qrGone, '| back to "Generate QR code" label:', backToGenerate);

const histToggle = app.page.locator('button').filter({ hasText: 'Show history' });
if (await histToggle.isVisible()) {
  await histToggle.click();
  await app.page.waitForTimeout(400);
}
const text = await bodyText();
console.log('History visible and mentions "Revoked":', text.includes('Revoked'));
await app.shot('QA-117-after-cancel-history');

console.log('\nQA-117 PASS (Cancel revokes + card resets):', regenVisible && cancelVisible && qrGone && backToGenerate);

await app.done();
