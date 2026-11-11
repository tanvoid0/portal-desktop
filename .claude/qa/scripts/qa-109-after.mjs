import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.screenshot({ path: '.claude/qa/shots/QA-109-after-switch-track-visible.png', clip: { x: 280, y: 390, width: 700, height: 110 } });

// Sanity sweep: other controls relying on --input (Input, Textarea, Button outline, Radio, OTP).
await app.shot('QA-109-after-devices-full-page');
await app.done();
