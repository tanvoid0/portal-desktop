import { open } from './i2.mjs';
const app = await open('sdk/managers', { width: 1280, height: 800 });
const page = app.page;
const warm = async () => { await page.waitForTimeout(8000); await page.evaluate(() => { const a=[...document.querySelectorAll('a[href="/sdk"]')].pop(); a.click(); }); await page.waitForTimeout(3500); };
await warm();
const u = () => page.url().replace('http://localhost:1421','');
const mainBtns = async () => page.evaluate(()=>[...document.querySelectorAll('main button')].map(b=>b.innerText.trim().replace(/\n/g,' / ')));
console.log('overview main buttons:', JSON.stringify((await mainBtns()).slice(12)));
for (const label of ['SDK Managers','Software Installer','Services','Versions']) {
  const n = await page.evaluate((l) => {
    const bs = [...document.querySelectorAll('main button')].filter(b => b.innerText.trim() === l);
    if (!bs.length) return 'MISSING';
    bs[bs.length-1].click(); return 'clicked';
  }, label);
  await page.waitForTimeout(2500);
  console.log(`overview button "${label}": ${n} -> ${u()}`);
  if (n !== 'MISSING') { await page.goBack(); await page.waitForTimeout(2500); }
}
// quick nav manager tile
await page.evaluate(() => { const b=[...document.querySelectorAll('main button')].find(x=>/^Conda/.test(x.innerText.trim())); b.click(); });
await page.waitForTimeout(2500);
console.log('quick-nav Conda tile ->', u());
await page.goBack(); await page.waitForTimeout(2500);
await page.evaluate(() => { const b=[...document.querySelectorAll('main button')].find(x=>/^Python/.test(x.innerText.trim())); b.click(); });
await page.waitForTimeout(2500);
console.log('quick-nav Python SDK tile ->', u());
await app.done();
