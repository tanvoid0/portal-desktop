import { open } from './app.mjs';
const app = await open('devices');
const p = app.page;
await p.waitForTimeout(1500);
const wasDark = await p.evaluate(() => document.documentElement.classList.contains('dark'));
if (!wasDark) await p.evaluate(() => document.documentElement.classList.add('dark'));
await p.waitForTimeout(300);
const r = await p.evaluate(() => {
  const sw = [...document.querySelectorAll('[data-slot="switch"]')].find((e) => e.getAttribute('data-state') === 'unchecked');
  if (!sw) return null;
  const rgba = (css) => { const c = document.createElement('canvas').getContext('2d', { willReadFrequently: true }); c.clearRect(0,0,1,1); c.fillStyle = css; c.fillRect(0,0,1,1); const d = c.getImageData(0,0,1,1).data; return [...d]; };
  let bg = sw.parentElement; let card = [0,0,0,0];
  while (bg) { const a = rgba(getComputedStyle(bg).backgroundColor); if (a[3] === 255) { card = a; break; } bg = bg.parentElement; }
  const t = rgba(getComputedStyle(sw).backgroundColor);
  const al = t[3] / 255;
  const comp = t.slice(0, 3).map((v, i) => v * al + card[i] * (1 - al));
  const lum = (c) => { const f = c.map((v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126*f[0]+0.7152*f[1]+0.0722*f[2]; };
  const [a, b] = [lum(comp), lum(card)].sort((x, y) => y - x);
  sw.scrollIntoView({block:'center'});
  return { track: comp.map(Math.round), card: card.slice(0,3), contrast: ((a + 0.05) / (b + 0.05)).toFixed(2) };
});
console.log('dark was already on:', wasDark, JSON.stringify(r));
await app.shot('QA-109-recheck2-dark-devices');
if (!wasDark) await p.evaluate(() => document.documentElement.classList.remove('dark'));
await app.done();
