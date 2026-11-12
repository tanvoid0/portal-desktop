// usage: node qa-actions-run.mjs <projectId> <card title> <shotname> [waitSec]
import { open } from './app.mjs';
const [id, title, shot, wait = '120'] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines`);
const p = app.page;
await p.getByText('Local run history').first().waitFor({timeout:60000}); await p.waitForTimeout(3000);
const handle = await p.evaluateHandle((title) => { for (const b of document.querySelectorAll('button')) { if (b.innerText.trim() !== 'Run') continue; let a = b.parentElement; while (a && a.querySelectorAll('button').length < 2 && !a.innerText.includes(title)) a = a.parentElement; if (a && a.innerText.includes(title) && [...a.querySelectorAll('button')].filter(x=>x.innerText.trim()==='Run').length === 1) return b; } return null; }, title);
const btn = handle.asElement(); if (!btn) throw new Error('no run button for ' + title);

console.log('tag', await btn.evaluate((e) => e.tagName));
await btn.click();
await p.waitForTimeout(1200); console.log("immediately after click:", (await p.locator("body").innerText()).slice(0,0)); await app.shot(shot + "-immediately");
const t0 = Date.now();
let last = '';
while (Date.now() - t0 < Number(wait) * 1000) {
  await p.waitForTimeout(3000);
  const txt = await p.locator('body').innerText();
  const i = txt.indexOf('Local run history');
  const snap = txt.slice(i, i + 700).replace(/\n+/g, ' | ');
  if (snap !== last) { console.log(`[${Math.round((Date.now() - t0) / 1000)}s]`, snap); last = snap; }
  if (/exit|passed|failed|succe|Completed|Failed/i.test(snap) && !/No action runs yet/.test(snap) && !/running/i.test(snap)) break;
}
console.log(await app.shot(shot));
console.log('toasts:', await p.locator('[data-sonner-toast], [role="status"]').allInnerTexts());
await app.done();
