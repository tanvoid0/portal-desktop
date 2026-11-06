import { open } from './app.mjs';
const routes = ['general','updates','editor','terminal','ides','frameworks','frameworks-languages','languages','package-managers','github','ai','ai/training','autonomy','learning','theme'];
const app = await open('');
const page = app.page;
for (const r of routes) {
  const before = app.errors.length;
  await page.goto(new URL('/settings/' + r, page.url()).href);
  await page.waitForLoadState('load'); await page.waitForTimeout(1200);
  const info = await page.evaluate(() => {
    const unl = [...document.querySelectorAll('button,a,[role=button],input,select,textarea,[role=switch],[role=checkbox],[role=combobox]')].filter(e => {
      const n = (e.getAttribute('aria-label') || e.textContent || e.getAttribute('title') || e.getAttribute('placeholder') || '').trim();
      const lab = e.labels?.length || e.getAttribute('aria-labelledby');
      return !n && !lab;
    }).map(e => e.outerHTML.slice(0, 120));
    return { h1: document.querySelector('h1')?.textContent?.trim(), path: location.pathname, unl, text: document.body.innerText.slice(0,150).replace(/\n/g,' | '), ovf: document.documentElement.scrollWidth > innerWidth };
  });
  console.log('##', r, JSON.stringify(info));
  if (app.errors.length > before) console.log('  errors:', app.errors.slice(before));
  await app.shot('sweep-settings-' + r.replace('/','-') + '-light-1280');
}
await app.done();
