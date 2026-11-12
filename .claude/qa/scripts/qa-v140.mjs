import { open } from './app.mjs';
const app = await open('projects/14/pipelines'); const p = app.page;
for (const id of [14,4,12,19,10,15,176]) {
  await p.goto(new URL(`/projects/${id}/pipelines`, p.url()).href); await p.waitForTimeout(1500);
  await p.waitForFunction(() => !document.body.innerText.includes('Loading actions catalog'), null, {timeout:60000}).catch(()=>{});
  await p.waitForTimeout(1200);
  const r = await p.evaluate(() => { const out=[]; document.querySelectorAll('h3, [data-slot="card-title"], .font-semibold').forEach(()=>{}); const t=document.body.innerText; const a=t.indexOf('Workflows'); const b=t.indexOf('Generate a pipeline'); return t.slice(a<0?t.indexOf('Actions\n'):a,b).replace(/\n+/g,' | ').slice(0,700); });
  console.log(`#${id}:`, r);
  if (id===14||id===4) await app.shot(`QA-154-after-catalog-${id}`);
}
await app.done();
