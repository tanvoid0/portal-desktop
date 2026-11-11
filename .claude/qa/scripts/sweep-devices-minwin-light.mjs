// Sweep: Devices page at minimum window (1024x600) and light theme.
import { open } from './app.mjs';

const app = await open('devices', { width: 1024, height: 600 });
await app.page.waitForTimeout(500);
await app.shot('sweep-devices-minwin-dark');

// Flip to light theme via the sidebar's theme toggle (bottom-left moon/sun icon).
const themeToggle = app.page.locator('button').filter({ hasText: '' }).first();
// Find by icon title/aria more reliably: look for a button near the bottom with sun/moon.
const toggles = await app.page.locator('button[aria-label*=theme i], button[title*=theme i]').count();
console.log('theme toggle buttons found:', toggles);
if (toggles > 0) {
  await app.page.locator('button[aria-label*=theme i], button[title*=theme i]').first().click();
  await app.page.waitForTimeout(400);
  await app.shot('sweep-devices-minwin-light');
}

await app.done();
