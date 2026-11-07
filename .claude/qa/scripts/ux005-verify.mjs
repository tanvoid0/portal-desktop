// UX-005: settings nav height, no scroll, item labels, aria-current, h1 text per route.
import { open } from './i2.mjs';
const app = await open('settings/general', { width: 1280, height: 800 });
const page = app.page;

async function report(tag) {
  const m = await page.evaluate(() => {
    const nav = document.querySelector('nav, [data-settings-nav]');
    const links = [...document.querySelectorAll('aside a, nav a')].filter(a => (a.getAttribute('href')||'').startsWith('/settings'));
    const scrollers = [...document.querySelectorAll('div')].filter(d => d.scrollHeight > d.clientHeight + 2 && d.clientHeight > 100);
    const card = links[0] ? links[0].closest('[data-slot="card"], .rounded-xl, div') : null;
    const h1s = [...document.querySelectorAll('h1')].map(h => h.innerText.trim());
    const h2s = [...document.querySelectorAll('h2')].map(h => h.innerText.trim());
    return {
      navLinks: links.length,
      items: links.map(a => {
        const b = a.getBoundingClientRect();
        return { href: a.getAttribute('href'), text: a.innerText.trim().replace(/\n/g,'/'), h: Math.round(b.height), top: Math.round(b.top), bottom: Math.round(b.bottom), cur: a.getAttribute('aria-current') };
      }),
      navCardWidth: card ? Math.round(card.getBoundingClientRect().width) : null,
      lastBottom: links.length ? Math.round(links[links.length-1].getBoundingClientRect().bottom) : null,
      vh: window.innerHeight,
      scrollingContainers: scrollers.map(d => ({cls: d.className.toString().slice(0,60), sh: d.scrollHeight, ch: d.clientHeight})),
      h1s, h2s,
      badgeRow: /Configured:\s*\d+\/\d+/.test(document.body.innerText),
    };
  });
  console.log(tag, JSON.stringify(m, null, 1));
}
await report('[settings/general 1280x800]');
await app.shot('UX-005-after-settings-general-1280');
for (const r of ['settings', 'settings/ides', 'settings/ai/providers', 'settings/theme', 'settings/autonomy']) {
  await app.go(r);
  const h1 = await page.evaluate(() => [...document.querySelectorAll('h1')].map(h=>h.innerText.trim()));
  const cur = await page.evaluate(() => [...document.querySelectorAll('a[aria-current]')].map(a=>a.getAttribute('href')+':'+a.getAttribute('aria-current')));
  console.log(`/${r} -> h1=${JSON.stringify(h1)} aria-current=${JSON.stringify(cur)}`);
}
await app.go('settings/general');
await app.size(1024, 600);
await report('[settings/general 1024x600]');
await app.shot('UX-005-after-settings-general-1024x600');
await app.size(1280, 800);
await app.done();
