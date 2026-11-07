import { open } from './i2.mjs';
const app = await open('sdk', { width: 1280, height: 800 });
const page = app.page;
const url = () => page.url().replace('http://localhost:1421','');

// overview nav buttons
for (const name of ['SDK Managers', 'Versions', 'Services', 'Overview']) {
  await page.goto('http://localhost:1421/sdk'); await page.waitForTimeout(1300);
  const b = page.getByRole('button').filter({ hasText: new RegExp('^'+name) }).last();
  const tag = await b.evaluate(e => e.tagName).catch(()=>'MISSING');
  await b.click();
  await page.waitForTimeout(1500);
  console.log(`overview "${name}" (tag ${tag}) -> ${url()}`);
}
// software installer button
await page.goto('http://localhost:1421/sdk'); await page.waitForTimeout(1300);
const all = await page.evaluate(()=>[...document.querySelectorAll('button,a')].map(b=>b.tagName+':'+b.innerText.trim().replace(/\n/g,' / ')).filter(t=>/install/i.test(t)));
console.log('installer-ish controls:', JSON.stringify(all));
// quick navigation manager tile
const tiles = await page.evaluate(()=>{ const h=[...document.querySelectorAll('*')].find(e=>e.children.length===0 && /Quick Navigation/i.test(e.textContent)); return h ? [...h.closest('div').parentElement.querySelectorAll('button,a')].map(b=>b.tagName+':'+b.innerText.trim().replace(/\n/g,' / ')).slice(0,12) : 'NO QUICK NAV'; });
console.log('quick nav:', JSON.stringify(tiles));
await app.shot('UX-014-after-sdk-overview-full');
await app.done();
