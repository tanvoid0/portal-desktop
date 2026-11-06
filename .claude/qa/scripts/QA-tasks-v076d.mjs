import { open } from './app.mjs';
const app = await open('tasks'); const p = app.page;
await p.setViewportSize({width:1024,height:600});
for (const r of ['tasks/create','tasks/67','tasks/67/edit','tasks/generate']) {
  await p.goto(new URL('/'+r, p.url()).href); await p.waitForTimeout(1200);
  const m = await p.evaluate(()=>{const els=[...document.querySelectorAll('div')].filter(e=>e.scrollHeight>e.clientHeight+5&&/auto|scroll/.test(getComputedStyle(e).overflowY));const e=els[els.length-1];if(!e)return {noscroller:true,docH:document.documentElement.scrollHeight};e.scrollTop=1e6;return {sh:e.scrollHeight,ch:e.clientHeight,top:e.scrollTop}});
  const btns = await p.evaluate(()=>[...document.querySelectorAll('button')].filter(b=>/Create Task|Update|Save|Generate|Delete/i.test(b.textContent)).map(b=>[b.textContent.trim().slice(0,20),Math.round(b.getBoundingClientRect().bottom),innerHeight]));
  console.log(r, JSON.stringify(m), JSON.stringify(btns));
  await app.shot('QA-076-after-route-'+r.replace(/\//g,'_')+'-bottom');
}
await app.done();
