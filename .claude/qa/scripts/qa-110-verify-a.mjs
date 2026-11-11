// QA-110 verify (a): generating a QR should NOT bump "Paired" or show a ghost card.
import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);

const statsBefore = await app.page.evaluate(() => document.body.innerText.match(/Paired\s+(\d+)/)?.[1]);
console.log('Paired count before:', statsBefore);

const genBtn = app.page.locator('button').filter({ hasText: 'Generate QR code' });
await genBtn.click();
await app.page.waitForTimeout(1500);
await app.shot('QA-110-after-qr-generated-no-ghost');

const statsAfter = await app.page.evaluate(() => document.body.innerText.match(/Paired\s+(\d+)/)?.[1]);
console.log('Paired count after QR generated:', statsAfter);

await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);
const text = await app.page.evaluate(() => document.body.innerText);
console.log('mentions "Unnamed device":', text.includes('Unnamed device'));
await app.shot('QA-110-after-qr-generated-paired-list');

await app.done();
