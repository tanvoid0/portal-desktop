import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

// Try CDP-level viewport resize to approximate the minimum window.
try {
  await app.page.setViewportSize({ width: 1024, height: 600 });
  console.log('viewport resize: ok');
} catch (e) {
  console.log('viewport resize failed:', e.message);
}
await app.page.waitForTimeout(300);
await app.shot('sweep-devices-minwin-dark');

// Locate the theme toggle near the bottom of the sidebar (moon/sun icon button, no visible text).
const sidebarButtons = app.page.locator('aside button, nav button');
const count = await sidebarButtons.count();
console.log('sidebar button count:', count);
for (let i = 0; i < count; i++) {
  const b = sidebarButtons.nth(i);
  const txt = (await b.innerText().catch(() => '')).trim();
  const aria = await b.getAttribute('aria-label').catch(() => null);
  if (!txt && (aria || true)) {
    console.log(i, 'aria=', aria, 'html snippet=', (await b.innerHTML()).slice(0, 80));
  }
}

await app.done();
