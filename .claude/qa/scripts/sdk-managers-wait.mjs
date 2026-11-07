import { open } from './i2.mjs';
const app = await open('sdk/managers', { width: 1280, height: 800 });
const page = app.page;
for (const t of [3,8,18,33]) {
  await page.waitForTimeout(t===3?2000:5000+(t*300));
  const b = await page.innerText('body');
  console.log(`t~${t}s loading=${/Loading/.test(b)} rows=${await page.evaluate(()=>document.querySelectorAll('table tbody tr, [data-manager-row]').length)} snippet=${b.slice(b.indexOf('SDK Managers'), b.indexOf('SDK Managers')+220).replace(/\n/g,' | ')}`);
}
await app.shot('sdk-managers-1280');
await app.done();
