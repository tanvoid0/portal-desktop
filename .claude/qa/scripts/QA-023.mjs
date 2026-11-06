// QA-023: /ai/coder at 1024x600: header wraps, composer pushed below the fold, workspace path select clipped.
import { open } from './app.mjs';
const app = await open('ai/coder', { width: 1024, height: 600 });
await app.page.setViewportSize({ width: 1024, height: 600 }).catch(() => {});
await app.page.waitForTimeout(1500);
const ta = app.page.locator('textarea').first();
const b = await ta.boundingBox();
console.log('composer bottom y =', Math.round(b.y + b.height), 'viewport h = 600 ->', '(textarea bottom is not the composer bottom; check the shot: chips and send button are cut off)');
await app.shot('QA-023-before-coder-min-window');
await app.done();
