import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.screenshot({ path: '.claude/qa/shots/sweep-devices-switch-zoom2.png', clip: { x: 280, y: 390, width: 700, height: 110 } });
await app.done();
