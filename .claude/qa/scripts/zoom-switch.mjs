import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
const el = app.page.locator('text=Pair a phone').locator('xpath=..').first();
await el.screenshot({ path: '.claude/qa/shots/sweep-devices-switch-zoom.png' });
await app.done();
