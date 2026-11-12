// usage: CDP_PORT=9223 node qa-diag-full.mjs <projectId> <actionTitle> <shotPrefix>
// ONE webview context, no goto after start: run action -> Diagnose & fix -> SPA to Coder, reject shell approvals -> SPA back.
import { open } from './app.mjs';
const [id, title, pre] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines`);
const p = app.page;
p.on('framenavigated', (f) => f === p.mainFrame() && console.log('NAV', f.url()));
await p.getByText('Local run history').first().waitFor({ timeout: 60000 });
await p.waitForTimeout(2500);
const stamp = Date.now(); await p.evaluate((s) => (window.__qaMarker = s), stamp);
const marker = async () => ((await p.evaluate(() => window.__qaMarker).catch(() => null)) === stamp ? 'same' : 'LOST');
const h = await p.evaluateHandle((title) => { for (const b of document.querySelectorAll('button')) { if (b.innerText.trim() !== 'Run') continue; let a = b.parentElement; while (a && !(a.innerText.trimStart().startsWith(title) && [...a.querySelectorAll('button')].filter(x=>x.innerText.trim()==='Run').length===1)) a = a.parentElement; if (a) return b; } return null; }, title);
await h.asElement().click();
await p.getByText('Run failed').first().waitFor({ timeout: 180000 });
await p.getByText('Run failed').first().scrollIntoViewIfNeeded();
const cardTxt = () => p.evaluate(() => { const e=[...document.querySelectorAll('div')].find(x=>/^Run failed/.test(x.innerText||'')&&x.innerText.length<2500&&x.querySelector('button')); return e?e.innerText.replace(/\n+/g,' | '):null; });
console.log('CARD:', await cardTxt()); console.log(await app.shot(pre + '-failure-card'));
await p.locator('button').filter({ hasText: 'Diagnose & fix' }).first().click();
const t0 = Date.now(); let last='';
const snap = async () => { const c = await cardTxt(); const amber = await p.getByText('Portal default needs a fix').count(); const s = `marker=${await marker()} amber=${amber} ${c}`; if (s!==last){ console.log(`[${Math.round((Date.now()-t0)/1000)}s]`, s.slice(0,600)); last=s; } return c; };
for (let i=0;i<8;i++){ await p.waitForTimeout(5000); await snap(); }
console.log(await app.shot(pre + '-diagnosing'));
// to Coder via SPA links
await p.locator('a[href="/ai"]').first().click(); await p.waitForTimeout(1500);
const coderLink = p.locator('a[href="/ai/coder"]').first(); if (await coderLink.count()) { await coderLink.click(); await p.waitForTimeout(2000); }
console.log('url', p.url(), 'marker', await marker());
await p.getByPlaceholder('Search sessions...').fill('A Portal Action'); await p.waitForTimeout(800);
const bodyBefore = await p.locator('body').innerText();
await p.getByText(/A Portal Action command/).first().click(); await p.waitForTimeout(2000);
for (let i=0;i<40;i++){
  const rej = p.locator('button').filter({ hasText: /^Reject$/ });
  if (await rej.count()) { const t=(await p.locator('body').innerText()); const j=t.lastIndexOf('Approval required'); console.log('approval:', t.slice(j,j+60).replace(/\n+/g,' | ')); await rej.first().click(); }
  await p.waitForTimeout(8000);
  const t = await p.locator('body').innerText();
  if (/DOCTOR_VERDICT: (app|pipeline|environment|flaky)\s*\n/.test(t.split('Worked for')[1] ?? '')) { console.log('thread finished'); break; }
}
console.log(await app.shot(pre + '-thread-done'));
await p.goBack(); await p.waitForTimeout(1500); await p.goBack().catch(()=>{}); await p.waitForTimeout(1500);
console.log('back url', p.url(), 'marker', await marker());
for (let i=0;i<24;i++){ await p.waitForTimeout(5000); const c = await snap(); if (/Fixed|Gave up/.test(c ?? '') || c===null) break; }
console.log(await app.shot(pre + '-final'));
console.log((await p.locator('body').innerText()).split('Generate a pipeline')[0].slice(-200));
await app.done();
