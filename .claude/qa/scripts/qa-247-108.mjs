import { open } from './app.mjs';
const app = await open('agents/run');
const p = app.page;
const nr = p.locator('button').filter({ hasText: 'New run' }).first();
if (await nr.isVisible().catch(() => false)) { await nr.click(); await p.waitForTimeout(500); }
await p.evaluate(()=>sessionStorage.removeItem('sveltekit:snapshot'));
await p.route('**://ipc.localhost/agents_list_backends', (r) => r.fulfill({ status: 500, body: 'QA-108 simulated failure' }));
await p.reload(); await p.waitForLoadState('load'); await p.waitForTimeout(2500);
const t = await p.evaluate(() => document.body.innerText);
console.log('has [object ArrayBuffer]:', /object ArrayBuffer/.test(t));
console.log('error text:', JSON.stringify(t.match(/Something went wrong[\s\S]{0,80}/)?.[0] ?? t.slice(0, 300)));
await app.shot('QA-108-recheck2-forced-error');
await p.unroute('**://ipc.localhost/agents_list_backends');
const retry = p.locator('button').filter({ hasText: /retry|try again/i });
if (await retry.count()) { await retry.first().click(); await p.waitForTimeout(1500); console.log('recovered, select count:', await p.locator('button[role=combobox]').count()); }
await app.done();
