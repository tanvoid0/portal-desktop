// Repro: AI chat panel header on the generate pages overflows its column (Provider select cut off, horizontal scrollbar).
import { open } from './app.mjs';
for (const route of ['tasks/generate', 'documents/generate']) {
  const app = await open(route);
  await app.page.waitForTimeout(1500);
  const r = await app.page.evaluate(() => {
    const sel = [...document.querySelectorAll('button,[role=combobox]')].find((e) => /Provider/.test(e.innerText));
    const box = sel?.getBoundingClientRect();
    const m = [...document.querySelectorAll('*')].find((e) => e.scrollWidth > e.clientWidth + 2 && getComputedStyle(e).overflowX !== 'visible' && e.clientWidth > 200);
    return { providerRight: box && Math.round(box.right), viewport: innerWidth, scrollerOverflow: m && `${m.scrollWidth}>${m.clientWidth}` };
  });
  console.log(route, JSON.stringify(r), '(expected no horizontal overflow)');
  await app.shot('QA-030-before-' + route.replace('/', '-') + '-chat-header');
  await app.done();
}
