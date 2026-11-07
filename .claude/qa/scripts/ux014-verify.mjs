import { open } from './i2.mjs';
const app = await open('sdk', { width: 1280, height: 800 });
const page = app.page;
const go = async (r) => { await page.goto('http://localhost:1421'+r); await page.waitForLoadState('load'); await page.waitForTimeout(1400); return page.url().replace('http://localhost:1421',''); };
for (const r of ['/sdk/manager', '/sdk/manager/nvm', '/sdk?tab=managers', '/sdk?tab=services', '/sdk?tab=versions', '/sdk?tab=overview', '/sdk/managers', '/sdk/versions', '/sdk/services', '/sdk/installations']) {
  const landed = await go(r);
  const h1 = await page.evaluate(() => [...document.querySelectorAll('h1')].map(h=>h.innerText.trim()));
  const body404 = (await page.innerText('body')).includes('404');
  console.log(`${r}  ->  ${landed}   h1=${JSON.stringify(h1)}${body404?'  [404]':''}`);
}
await go('/sdk');
console.log('tabs on /sdk:', await page.evaluate(()=>[...document.querySelectorAll('[role="tab"]')].map(t=>t.innerText.trim())));
console.log('header buttons:', await page.evaluate(() => {
  const h = document.querySelector('h1')?.closest('div')?.parentElement;
  return [...document.querySelectorAll('button')].slice(0,30).map(b=>b.innerText.trim()).filter(Boolean);
}));
await app.shot('UX-014-after-sdk-overview-1280');
await app.done();
