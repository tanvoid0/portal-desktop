import { open } from './app.mjs';
const app = await open('tasks?project=all&view=list'); const p = app.page;
await p.setViewportSize({width:1024,height:600}); await p.waitForTimeout(700);
const sc = p.locator('div.overflow-y-auto').last();
console.log('sw/cw', await sc.evaluate(e=>[e.scrollWidth,e.clientWidth,getComputedStyle(e).overflowX]));
const el = p.getByText(/^QA Nospace/).first(); await el.scrollIntoViewIfNeeded(); await p.waitForTimeout(300);
console.log('title', JSON.stringify(await el.evaluate(e=>{const r=e.getBoundingClientRect();return [Math.round(r.left),Math.round(r.right),e.scrollWidth>e.clientWidth,e.className.slice(0,80)]})));
// right edge of the row's badges
console.log('badges', await el.evaluate(e=>{let row=e;for(let i=0;i<6;i++){row=row.parentElement; if(row.getBoundingClientRect().width>600)break;} const b=[...row.querySelectorAll('*')].filter(x=>/^(Completed|To Do|In Progress|Cancelled)$/.test(x.textContent.trim())&&x.children.length==0)[0]; const r=b?.getBoundingClientRect(); return r?[Math.round(r.left),Math.round(r.right),innerWidth]:'nobadge'}));
await app.shot('QA-077-after-nospace-1024x600-v2');
await app.done();
