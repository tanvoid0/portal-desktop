// Vault unlock dialog must be centred in the viewport from first paint, at several viewport sizes/zooms.
import { open } from './app.mjs';
const app = await open('');
const { page } = app;
const sizes = [[1280, 800], [1024, 600], [800, 500], [2560, 1080]];
let bad = 0;
for (const [w, h] of sizes) {
  await page.setViewportSize({ width: w, height: h }).catch(() => {});
  await page.evaluate(() => {
    window.__samples = [];
    const loop = () => {
      const el = document.querySelector('[data-slot="dialog-content"]');
      if (el) { const r = el.getBoundingClientRect(); window.__samples.push([Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2), innerWidth, innerHeight]); }
      if (window.__samples.length < 120) requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
    window.__TAURI_INTERNALS__.invoke('plugin:event|emit', { event: 'vault://unlock-requested' });
  });
  await page.waitForTimeout(2500);
  const s = await page.evaluate(() => window.__samples);
  const off = s.filter(([cx, cy, iw, ih]) => Math.abs(cx - iw / 2) > 2 || Math.abs(cy - ih / 2) > 2);
  console.log(`${w}x${h}: ${s.length} samples, off-centre ${off.length}`, off[0] ?? '', s[0] ?? '');
  if (off.length) bad++;
  await page.keyboard.press('Escape');
  await page.waitForTimeout(500);
}
console.log(bad ? 'BUG: dialog off-centre' : 'ok');
await app.done();
