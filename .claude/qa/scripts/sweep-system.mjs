// /system + sidebar host stats sweep. Dark mode is toggled by class only (nothing persisted).
import { open } from './app.mjs';
const app = await open('system', {});
const { page } = app;
await page.waitForTimeout(9000);
const theme = (dark) => page.evaluate((d) => document.documentElement.classList.toggle('dark', d), dark);
const wasDark = await page.evaluate(() => document.documentElement.classList.contains('dark'));
const probe = () =>
  page.evaluate(() => {
    const over = [...document.querySelectorAll('*')]
      .filter((e) => e.scrollWidth > e.clientWidth + 1 && /auto|scroll/.test(getComputedStyle(e).overflowX))
      .map((e) => `${e.tagName}.${String(e.className).slice(0, 60)} ${e.scrollWidth}>${e.clientWidth}`);
    return {
      win: innerWidth + 'x' + innerHeight,
      overflowX: over,
      tables: [...document.querySelectorAll('table')].map((t) =>
        [...t.rows].map((r) => [...r.cells].filter((c) => c.offsetParent).map((c) => c.innerText.trim()).join(' | '))
      ),
    };
  });
const state = () =>
  page.evaluate(() => ({
    lines: [...document.querySelectorAll('polyline')].map((p) => p.getAttribute('points').split(' ').length + ':' + p.getAttribute('points').slice(-30)),
    cores: [...document.querySelectorAll('[role=img][aria-label^="Per-core"] > div > div')].map((d) => d.style.height).join(','),
    cards: [...document.querySelectorAll('h2')].map((h) => h.parentElement.innerText.replace(/\n+/g, ' / ').slice(0, 220)),
  }));
console.log(JSON.stringify(await state(), null, 1));
await page.waitForTimeout(4200);
console.log(JSON.stringify(await state(), null, 1));
for (const [name, w, h] of [['narrow', 1024, 600], ['wide', 2560, 1300], ['mid', 1280, 800]]) {
  await page.setViewportSize({ width: w, height: h });
  for (const dark of [false, true]) {
    await theme(dark);
    await page.waitForTimeout(600);
    const t = dark ? 'dark' : 'light';
    console.log(await app.shot(`sweep-system-${name}-${t}`));
    // second shot with a tall window so below-fold content is in frame
    await page.setViewportSize({ width: w, height: 2000 });
    await page.waitForTimeout(500);
    console.log(await app.shot(`sweep-system-${name}-${t}-full`));
    await page.setViewportSize({ width: w, height: h });
  }
  console.log(name, JSON.stringify(await probe(), null, 1));
}
await theme(wasDark);
await app.done();
