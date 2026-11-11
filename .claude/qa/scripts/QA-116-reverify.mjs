// Re-verify QA-116 on instance-2 (CDP :9223): generating a pairing QR must not
// immediately (or ever, unscanned) count as "Paired".
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function pairedCount() {
  const text = await app.page.evaluate(() => document.body.innerText);
  const m = text.match(/Paired\s+(\d+)/);
  return m ? Number(m[1]) : null;
}

const before = await pairedCount();
console.log('Paired count before QR:', before);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(1500);
await app.shot('QA-116-reverify-qr-generated');

const after = await pairedCount();
console.log('Paired count after QR (unscanned):', after);
console.log('RESULT QA-116 holds (count unchanged):', before === after);

const bodyText = await app.page.evaluate(() => document.body.innerText);
console.log('Ghost "Unnamed device" visible in page text:', /Unnamed device/i.test(bodyText));

await app.done();
