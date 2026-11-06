import { open } from './app.mjs';
const app = await open('tasks?project=all&view=list'); const p = app.page;
const scrollers = () => p.evaluate(()=>{
  // find elements that actually scroll vertically inside main
  return [...document.querySelectorAll('main *, main')].filter(e=>{const s=getComputedStyle(e);return /(auto|scroll)/.test(s.overflowY)&&e.scrollHeight>e.clientHeight+2}).map(e=>({c:(e.className||'').toString().slice(0,60),sh:e.scrollHeight,ch:e.clientHeight}));
});
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({width:w,height:h}); await p.waitForTimeout(600);
  for (const v of ['list','archive']) {
    await p.goto(new URL(`/tasks?project=all&view=${v}`, p.url()).href); await p.waitForTimeout(1200);
    const sc = await scrollers();
    // scroll biggest to bottom & report last element visibility
    const r = await p.evaluate(async()=>{
      const c=[...document.querySelectorAll('main *')].filter(e=>/(auto|scroll)/.test(getComputedStyle(e).overflowY)&&e.scrollHeight>e.clientHeight+2).sort((a,b)=>b.scrollHeight-a.scrollHeight)[0];
      if(!c) return {none:true,docH:document.documentElement.scrollHeight,vh:innerHeight};
      c.scrollTop=1e6; await new Promise(r=>setTimeout(r,200));
      const rows=[...c.querySelectorAll('[role=button]')]; const last=rows[rows.length-1]; const lr=last?.getBoundingClientRect(), cr=c.getBoundingClientRect();
      return {st:c.scrollTop,max:c.scrollHeight-c.clientHeight,lastBottom:Math.round(lr?.bottom),contBottom:Math.round(cr.bottom),vh:innerHeight,docH:document.documentElement.scrollHeight};
    });
    console.log(`${w}x${h} ${v}`, JSON.stringify(sc), JSON.stringify(r));
    if (w===1280) await app.shot(`sweep-tasks-${v}-bottom`);
  }
}
// QA-077
await p.setViewportSize({width:1280,height:800});
await p.goto(new URL('/tasks?project=all&view=list', p.url()).href); await p.waitForTimeout(1200);
const row = p.locator('[role=button]', {hasText:'QA Nospace'}).first();
const m = await row.evaluate(e=>{const r=e.getBoundingClientRect();const pr=e.parentElement.getBoundingClientRect();return {right:Math.round(r.right),parentRight:Math.round(pr.right),sw:e.scrollWidth,cw:e.clientWidth,vw:innerWidth,badgesInView:[...e.querySelectorAll('span,div')].filter(x=>/^(High|Pending|To Do|HIGH)$/i.test(x.textContent.trim())).map(x=>Math.round(x.getBoundingClientRect().right))}});
console.log('QA-077', JSON.stringify(m));
await app.shot('QA-077-after-list-nospace-title');
await p.setViewportSize({width:1024,height:600}); await p.waitForTimeout(500);
await app.shot('QA-077-after-list-nospace-title-1024');
// breadcrumb
await p.setViewportSize({width:1280,height:800});
await p.goto(new URL('/tasks', p.url()).href); await p.waitForTimeout(1000);
const bc = async()=> (await p.locator('header, nav').first().innerText()).replace(/\s+/g,' ').slice(0,120);
await p.getByRole('button',{name:/List/}).first().click(); await p.waitForTimeout(500);
console.log('bc after list:', p.url(), '|', await bc());
await p.getByText('portal_desktop',{exact:false}).first().click().catch(()=>{}); await p.waitForTimeout(500);
console.log('bc after project:', p.url(), '|', await bc());
await app.shot('sweep-tasks-breadcrumb-after-switch');
await app.done();
