import { open } from './i2.mjs';
const app = await open('sdk', { width: 1280, height: 800 });
const page = app.page;
for (const t of [5,10,20,30,45]) {
  await page.waitForTimeout(t===5?4000:(t===10?5000:(t===20?10000:(t===30?10000:15000))));
  const s = await page.evaluate(() => {
    const b = document.body.innerText;
    return { loading: b.includes('Loading SDK data'), stats: (b.match(/Managers installed[\s\S]{0,120}/)||[''])[0].replace(/\n/g,' | '), quick: b.includes('Quick Navigation') };
  });
  console.log(`t=${t}s`, JSON.stringify(s));
  if (!s.loading) break;
}
await app.shot('sdk-overview-after-wait');
console.log('console errors so far:', app.errors.slice(0,6));
await app.done();
