import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const routes = process.argv[2].split(',');
const app = await open('');
const p = app.page;
for (const r of routes) {
  await p.goto(new URL('/' + r, p.url()).href); await dismissVault(p, 1500); await p.waitForTimeout(800);
  await p.evaluate(() => document.activeElement?.blur()); 
  const bad = []; let seen = new Set();
  for (let i = 0; i < 45; i++) {
    await p.keyboard.press('Tab');
    const s = await p.evaluate(() => { const e = document.activeElement; if (!e || e === document.body) return null; const cs = getComputedStyle(e); const vis = (cs.outlineStyle !== 'none' && parseFloat(cs.outlineWidth) > 0) || (cs.boxShadow && cs.boxShadow !== 'none'); const r = e.getBoundingClientRect(); return { id: (e.getAttribute('aria-label') || e.textContent || e.placeholder || e.tagName).trim().slice(0, 30), vis, tag: e.tagName, off: r.width === 0 || r.bottom < 0 || r.top > innerHeight, cls: e.className?.toString().slice(0,0) }; });
    if (!s) continue; const k = s.id + s.tag; if (seen.has(k)) continue; seen.add(k);
    if (!s.vis) bad.push(s.id + '<' + s.tag + '>');
  }
  console.log(r, 'tabbed', seen.size, 'no-visible-focus:', bad.slice(0, 12));
}
await app.done();
