import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('');
const p = app.page;
await dismissVault(p);
await app.shot('sweep-home-light-settled');
const snap = () => p.evaluate(() => ({ debug: /DEBUG/.test(document.body.innerText), t: document.body.innerText.replace(/\s+/g,' ').slice(0,400), url: location.href }));
console.log('settled', JSON.stringify(await snap()));
const variants = new Map();
for (let run = 0; run < 6; run++) {
  await p.goto(new URL('/', p.url()).href);
  const t0 = Date.now();
  while (Date.now() - t0 < 3500) { const s = await snap(); const k = (s.debug?'D':'-') + (s.t.match(/\b\d+\s*(Projects|projects|Tasks|tasks)\b/g)||[]).join(','); if(!variants.has(k)) variants.set(k, s.t.slice(0,250)); await p.waitForTimeout(40); }
  await dismissVault(p, 300);
}
console.log([...variants]);
await app.done();
