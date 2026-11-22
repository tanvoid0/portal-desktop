import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('projects/233?tab=actions');
const p = app.page;
await dismissVault(p);
await p.waitForTimeout(3000);
console.log(await app.shot('sweep-projects-233-actions-catalog-loaded'));
for (const t of ['History', 'Generate', 'Release', 'GitHub runs']) {
  await p.getByRole('tab', { name: new RegExp('^'+t) }).click().catch(()=>p.getByRole('button',{name:new RegExp('^'+t)}).first().click());
  await p.waitForTimeout(2500);
  console.log(await app.shot('sweep-projects-233-actions-' + t.toLowerCase().replace(' ','-')));
}
await app.done();
