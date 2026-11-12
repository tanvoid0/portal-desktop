// usage: CDP_PORT=9223 node qa-doctor-diagnose.mjs <projectId> <actionTitle> <shotPrefix> [maxSec]
// Runs a default action (build/test/lint only!), waits for the failure card, clicks Diagnose & fix, polls.
import { open } from './app.mjs';
const [id, title, pre, maxSec = '600'] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines`);
const p = app.page;
await p.getByText('Local run history').first().waitFor({ timeout: 60000 });
await p.waitForTimeout(3000);
const h = await p.evaluateHandle((title) => { for (const b of document.querySelectorAll('button')) { if (b.innerText.trim() !== 'Run') continue; let a = b.parentElement; while (a && !(a.innerText.includes(title) && [...a.querySelectorAll('button')].filter(x=>x.innerText.trim()==='Run').length===1)) a = a.parentElement; if (a) return b; } return null; }, title);
const btn = h.asElement(); if (!btn) throw new Error('no run button for ' + title);
await btn.click();
const t0 = Date.now();
await p.getByText('Run failed').first().waitFor({ timeout: 180000 });
console.log(`failure card after ${Math.round((Date.now()-t0)/1000)}s`);
await p.getByText('Run failed').first().scrollIntoViewIfNeeded();
const cardTxt = () => p.evaluate(() => { const e=[...document.querySelectorAll('div')].find(x=>/^Run failed/.test(x.innerText||'')&&x.innerText.length<2000&&x.querySelector('button')); return e?e.innerText.replace(/\n+/g,' | '):null; });
console.log('CARD:', await cardTxt());
console.log(await app.shot(pre + '-failure-card'));
const dbtn = p.locator('button').filter({ hasText: 'Diagnose & fix' }).first();
console.log('diagnose tag', await dbtn.evaluate(e=>e.tagName), 'disabled', await dbtn.isDisabled());
await dbtn.click();
let last = '';
const t1 = Date.now();
while (Date.now() - t1 < Number(maxSec) * 1000) {
  await p.waitForTimeout(5000);
  const c = await cardTxt();
  const amber = await p.getByText('Portal default needs a fix').count();
  const snap = `${c} || amberCard=${amber}`;
  if (snap !== last) { console.log(`[${Math.round((Date.now()-t1)/1000)}s]`, snap); last = snap; await app.shot(pre + '-progress'); }
  if (/Fixed|Gave up/.test(c ?? '') || (c === null)) break;
}
console.log(await app.shot(pre + '-final'));
console.log('toasts:', await p.locator('[data-sonner-toast]').allInnerTexts());
await app.done();
