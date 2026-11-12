// usage: CDP_PORT=9223 node qa-generate.mjs <projectId> "<requirement>" <shotPrefix>
// One webview context end to end (no goto after start): Generate -> SPA to Coder -> reject shell approvals -> back.
import { open } from './app.mjs';
const [id, req, pre] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines`);
const p = app.page;
p.on('framenavigated', (f) => f === p.mainFrame() && console.log('NAV', f.url()));
await p.getByText('Local run history').first().waitFor({ timeout: 60000 });
await p.waitForTimeout(2500);
const stamp = Date.now();
await p.evaluate((s) => (window.__qaMarker = s), stamp);
const ta = p.getByPlaceholder(/e\.g\. Add a lint/);
await ta.fill(req);
const gen = p.getByRole('button', { name: 'Generate', exact: true });
console.log('generate tag', await gen.evaluate((e) => e.tagName));
await gen.click();
const card = () => p.evaluate(() => { const h=[...document.querySelectorAll('*')].find(x=>x.children.length===0&&x.textContent.trim()==='Generate a pipeline'); const c=h?.closest('[class*="card"], [data-slot="card"]')||h?.parentElement?.parentElement?.parentElement; return c?c.innerText.replace(/\n+/g,' | '):'no card'; });
const marker = () => p.evaluate(() => window.__qaMarker).catch(()=> 'ctx-dead');
for (let i = 0; i < 6; i++) { await p.waitForTimeout(5000); console.log(`[${(i+1)*5}s] marker=${await marker()===stamp?'same':'LOST'} |`, (await card()).slice(0,400)); }
console.log(await app.shot(pre + '-generating'));
await app.done();
