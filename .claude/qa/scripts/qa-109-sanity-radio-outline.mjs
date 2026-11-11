import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 600);
await app.page.waitForTimeout(300);
await app.shot('QA-109-after-sanity-radio-and-outline-button');
await app.done();
