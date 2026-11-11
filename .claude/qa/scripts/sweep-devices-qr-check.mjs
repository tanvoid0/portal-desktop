import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);
await app.shot('sweep-devices-after-unscanned-qr-paired-list');
const text = await app.page.evaluate(() => document.body.innerText);
console.log(text.slice(text.indexOf('Paired devices'), text.indexOf('Paired devices') + 600));
await app.done();
