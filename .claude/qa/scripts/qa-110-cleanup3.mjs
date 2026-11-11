import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);
await app.shot('QA-110-cleanup3-initial-state');

// Dump every visible button's text so we know the real confirm-dialog label.
const buttons = await app.page.locator('button:visible').allInnerTexts();
console.log('visible buttons:', JSON.stringify(buttons));

await app.done();
