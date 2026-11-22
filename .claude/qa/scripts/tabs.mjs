import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('deployments');
const p = app.page;
await dismissVault(p);
await p.getByRole('tab', { name: /Deployments/ }).click().catch(()=>p.getByText('Deployments',{exact:true}).first().click());
await p.waitForTimeout(1500);
console.log(await app.shot('sweep-deployments-tab-deployments'));
for (const r of ['utilities/env','utilities/uninstaller','automation/workflows','automation/utilities']) {
  await p.goto(new URL('/' + r, p.url()).href); await dismissVault(p, 1500); await p.waitForTimeout(2500);
  console.log(r, '->', p.url());
  console.log(await app.shot('sweep-' + r.replace('/','-')));
}
await app.done();
