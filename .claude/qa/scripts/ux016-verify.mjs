import { open } from './i2.mjs';
const app = await open('scripts/new', { width: 1280, height: 800 });
const page = app.page;
const routes = ['scripts/new','projects/create','projects/edit/1','deployments/new','tasks/create','documents/create'];
for (const r of routes) {
  await app.go(r);
  await page.waitForTimeout(1500);
  const m = await page.evaluate(() => {
    const h1 = [...document.querySelectorAll('h1')].map(h=>h.innerText.trim());
    const h2 = [...document.querySelectorAll('h2')].map(h=>h.innerText.trim());
    const first = [...document.querySelectorAll('input,textarea,[role="combobox"]')].filter(e=>e.getBoundingClientRect().height>0)[0];
    return { h1, h2: h2.slice(0,4), firstFieldY: first? Math.round(first.getBoundingClientRect().top):null, mains: document.querySelectorAll('main').length,
             headerBtns: [...document.querySelectorAll('header button, [data-slot="page-header"] button')].map(b=>b.innerText.trim()).filter(Boolean).slice(0,6) };
  });
  console.log(`/${r}`, JSON.stringify(m));
}
await app.done();
