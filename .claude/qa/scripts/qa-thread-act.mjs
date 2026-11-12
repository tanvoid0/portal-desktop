// usage: qa-thread-act.mjs <Reject|Accept once|none> <shot>   (selects the doctor thread; NEVER use Accept on file writes)
import { open } from './app.mjs';
const [act, shot] = process.argv.slice(2);
const app = await open('ai/coder'); const p = app.page; await p.waitForTimeout(2500);
await p.getByPlaceholder('Search sessions...').fill('A Portal Action');
await p.waitForTimeout(800);
await p.getByText(/A Portal Action command/).first().click(); await p.waitForTimeout(2500);
const appr = p.locator('button').filter({ hasText: new RegExp('^'+act+'$') });
if (act !== 'none' && await appr.count()) { console.log('clicking', act); await appr.first().click(); await p.waitForTimeout(15000); }
const t = await p.locator('body').innerText(); const i = t.indexOf('DOCTOR_VERDICT'); console.log(t.slice(i, i+2500));
console.log(await app.shot(shot)); await app.done();
