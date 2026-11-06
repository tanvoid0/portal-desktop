// Icon-only buttons on automation/scripts, automation/blocks, scripts/new have no accessible name
import { open } from './app.mjs';
const app = await open('automation/scripts');
const p = app.page;
for (const r of ['automation/scripts','automation/blocks','scripts/new']) {
  await p.goto(new URL('/' + r, p.url()).href); await p.waitForTimeout(2500);
  const bad = await p.evaluate(() => {
    const c = {};
    for (const e of document.querySelectorAll('button')) {
      if (e.offsetParent === null) continue;
      if ((e.getAttribute('aria-label') || e.innerText || e.title || '').trim()) continue;
      const k = (e.querySelector('svg')?.getAttribute('class') || 'no-svg').match(/lucide-[a-z-]+/)?.[0] ?? 'svg';
      c[k] = (c[k] || 0) + 1;
    }
    return c;
  });
  console.log(r, JSON.stringify(bad));
}
await app.done();
