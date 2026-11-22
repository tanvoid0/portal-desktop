import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('tasks');
const p = app.page;
await dismissVault(p);
await p.waitForTimeout(1500);
console.log('168 rail:', await p.evaluate(()=>[...document.querySelectorAll('span[title]')].map(e=>e.title).filter(t=>/Project #|Unknown/.test(t))));
console.log(await app.shot('QA-168-after-tasks-rail'));
await p.getByText(/release_rehearse refuses/).first().click(); await p.waitForTimeout(900);
console.log('173 crumb:', await p.evaluate(()=>document.querySelector('nav[aria-label=breadcrumb],header')?.innerText.replace(/\s+/g,' ').slice(0,80)));
console.log(await app.shot('QA-173-after-breadcrumb'));
for (const [id,r] of [['174','automation/utilities'],['175','automation/blocks'],['175b','automation/workflows'],['177','database'],['180','utilities/disk'],['179','documents/create']]) {
  await p.goto(new URL('/'+r, p.url()).href); await dismissVault(p,1500); await p.waitForTimeout(2200);
  console.log(id, await app.shot(`QA-${id.replace('b','')}-after-${r.replace('/','-')}`));
}
console.log('179 archived checkbox on create:', await p.getByText('Archived (soft delete)').count());
await app.done();
