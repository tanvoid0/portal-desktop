import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);

const genBtn = app.page.locator('button').filter({ hasText: 'Generate QR code' });
console.log('generate button count:', await genBtn.count());
await genBtn.click();
await app.page.waitForTimeout(1500);
await app.shot('sweep-devices-qr-generated');

const bodyText = await app.page.evaluate(() => document.body.innerText);
console.log('mentions warning about no capabilities:', /no capabilit|at least one/i.test(bodyText));

await app.done();
