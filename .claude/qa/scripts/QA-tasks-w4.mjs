import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
await p.getByPlaceholder('Search tasks...').fill('QA ');
await p.waitForTimeout(500);
console.log(await p.locator('main').innerText().then(t=>t.slice(0,300)));
await app.shot('sweep-tasks-revamp-qa-fixtures');
// multi-select + long
await p.keyboard.press('Escape');
await p.getByRole('button',{name:'Multi-Select'}).click(); await p.waitForTimeout(300);
await app.shot('sweep-tasks-revamp-multiselect-long');
// geometry: card overflow
const bad = await p.evaluate(()=>{
  const out=[]; document.querySelectorAll('[role=button][aria-label^="Select task"], [draggable=true]').forEach(c=>{
    const r=c.getBoundingClientRect(); if(c.scrollWidth>c.clientWidth+1) out.push(['overflow',c.getAttribute('aria-label')?.slice(0,30),c.scrollWidth,c.clientWidth]);
  }); return out;});
console.log(bad);
await app.done();
