import { open } from './app.mjs';
const [id, req, pre] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines`); const p = app.page;
await p.getByText('Local run history').first().waitFor({ timeout: 60000 }); await p.waitForTimeout(2500);
await p.getByPlaceholder(/e\.g\. Add a lint/).fill(req);
await p.getByRole('button', { name: 'Generate', exact: true }).click();
const st = () => p.evaluate(() => { const t=document.body.innerText; const i=t.indexOf('Generate\n'); return t.slice(i, i+220).replace(/\n+/g,' | '); });
let last=''; const t0=Date.now();
while (Date.now()-t0 < 280000) { await p.waitForTimeout(4000); const s = await st(); if (s!==last){ console.log(`[${Math.round((Date.now()-t0)/1000)}s]`, s); last=s;} if (/Done|Failed/.test(s)) break; }
await p.waitForTimeout(1500);
console.log(await app.shot(pre)); await app.done();
