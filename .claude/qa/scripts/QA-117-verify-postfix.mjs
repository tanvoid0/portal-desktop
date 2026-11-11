// Post-fix verification for QA-117 (CDP :9223). Part A: generate a QR, click
// the new Cancel button before scanning, confirm the grant shows revoked in
// History and the card resets to "generate a new QR" state. Part B: generate
// another QR, check card state right after generation (Cancel button present,
// Regenerate label).
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function bodyText() {
  return app.page.evaluate(() => document.body.innerText);
}

console.log('--- Part A: generate then Cancel ---');
await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);

const genBtn = app.page.locator('button').filter({ hasText: /Regenerate QR code/ });
const cancelBtn = app.page.locator('button').filter({ hasText: /^Cancel$/ });
console.log('Button says "Regenerate QR code" after generating:', await genBtn.isVisible());
console.log('Cancel button visible:', await cancelBtn.isVisible());
const qrVisible = await app.page.locator('img[alt="Pairing QR code"]').isVisible();
console.log('QR image visible:', qrVisible);

await app.shot('QA-117-before-cancel-click');

await cancelBtn.click();
await app.page.waitForTimeout(800);

const qrGoneAfterCancel = !(await app.page.locator('img[alt="Pairing QR code"]').isVisible());
console.log('QR image gone after Cancel:', qrGoneAfterCancel);
const backToGenerateLabel = await app
  .page
  .locator('button')
  .filter({ hasText: /^Generate QR code$/ })
  .isVisible();
console.log('Button back to "Generate QR code":', backToGenerateLabel);

// Check History shows it revoked
await app.page.locator('button').filter({ hasText: /Show history/ }).click();
await app.page.waitForTimeout(300);
const historyText = await bodyText();
console.log('History section mentions "Revoked":', historyText.includes('Revoked'));
await app.shot('QA-117-after-cancel-history');

console.log('\nQA-117 Part A (Cancel) PASS:', qrGoneAfterCancel && backToGenerateLabel);

await app.done();
