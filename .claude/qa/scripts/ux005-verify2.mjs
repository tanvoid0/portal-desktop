import { open } from './i2.mjs';
const app = await open('settings/general', { width: 1280, height: 800 });
const page = app.page;

async function nav(tag) {
  const m = await page.evaluate(() => {
    const nav = document.querySelector('main nav') || [...document.querySelectorAll('nav')].pop();
    const btns = [...nav.querySelectorAll('button')];
    const navBox = nav.getBoundingClientRect();
    // nearest scrolling ancestor of the nav
    let sc = nav.parentElement; let scrolls = null;
    while (sc) { if (sc.scrollHeight > sc.clientHeight + 2) { scrolls = {cls: sc.className.toString().slice(0,70), sh: sc.scrollHeight, ch: sc.clientHeight}; break; } sc = sc.parentElement; }
    const card = nav.closest('[data-slot="card"]') || nav.parentElement;
    const cb = card.getBoundingClientRect();
    return {
      count: btns.length,
      heights: btns.map(b => Math.round(b.getBoundingClientRect().height)),
      labels: btns.map(b => b.innerText.trim()),
      navTop: Math.round(navBox.top), navBottom: Math.round(navBox.bottom),
      cardW: Math.round(cb.width), cardH: Math.round(cb.height), cardBottom: Math.round(cb.bottom),
      navScrollsInside: scrolls,
      ariaCurrent: [...document.querySelectorAll('[aria-current]')].map(e => e.innerText.trim() + ':' + e.getAttribute('aria-current')),
      vh: window.innerHeight,
    };
  });
  console.log(tag, JSON.stringify(m));
}
await nav('[1280x800 /settings/general]');
for (const r of ['settings/ai/providers','settings/languages','settings/ides','settings/autonomy']) {
  await app.go(r);
  const ac = await page.evaluate(() => [...document.querySelectorAll('[aria-current]')].map(e => e.innerText.trim()+':'+e.getAttribute('aria-current')));
  const h1 = await page.evaluate(() => document.querySelector('h1')?.innerText.trim());
  console.log(`/${r}: h1="${h1}" aria-current=${JSON.stringify(ac)}`);
}
await app.go('settings/general');
await app.size(1024, 600);
await nav('[1024x600 /settings/general]');
await app.shot('UX-005-after-settings-nav-1024x600');
await app.size(1440, 900);
await nav('[1440x900 /settings/general]');
await app.size(1280, 800);
await app.done();
