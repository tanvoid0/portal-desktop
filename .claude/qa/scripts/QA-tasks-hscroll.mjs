import { open } from './app.mjs';
const route = process.argv[2] || 'tasks?project=all&view=board';
const app = await open(route); const p = app.page;
const orig = p.viewportSize();
const widths = [1280, 1100, 1024, 900, 760];
async function measure(label){
  return p.evaluate(()=>{
    const root = document.querySelector('div.flex.gap-4.overflow-x-auto') || document.querySelector('[class*="overflow-x-auto"]');
    if(!root) return {err:'no board root'};
    const chain=[]; let el=root;
    while(el && el!==document.documentElement){
      const cs=getComputedStyle(el); const r=el.getBoundingClientRect();
      chain.push({tag:el.tagName.toLowerCase(), cls:(el.className||'').toString().slice(0,70), sw:el.scrollWidth, cw:el.clientWidth, ox:cs.overflowX, left:Math.round(r.left), right:Math.round(r.right), pr:cs.paddingRight});
      el=el.parentElement;
    }
    const cols=[...root.children].map(c=>{const r=c.getBoundingClientRect();return {w:Math.round(r.width),l:Math.round(r.left),r:Math.round(r.right)}});
    return {chain:chain.slice(0,9), cols, vw:innerWidth};
  });
}
for (const w of widths){
  await p.setViewportSize({width:w,height:800}); await p.waitForTimeout(600);
  const m = await measure();
  console.log('\n=== vw',w,'cols',JSON.stringify(m.cols));
  (m.chain||[]).forEach((c,i)=>console.log(i,c.tag,c.cls.slice(0,55).padEnd(55),'sw',c.sw,'cw',c.cw,c.ox,'x',c.left+'-'+c.right,'pr',c.pr));
  if (w===900||w===760){
    // try scrolling
    const r = await p.evaluate(()=>{const root=document.querySelector('div.flex.gap-4.overflow-x-auto'); if(!root) return null; root.scrollLeft=99999; const last=root.lastElementChild.getBoundingClientRect(); const rr=root.getBoundingClientRect(); return {scrollLeft:root.scrollLeft,max:root.scrollWidth-root.clientWidth,lastRight:Math.round(last.right),rootRight:Math.round(rr.right)}});
    console.log('scroll to end:',JSON.stringify(r));
    await app.shot(`sweep-tasks-hscroll-${w}`);
  }
}
if (orig) await p.setViewportSize(orig); else await p.setViewportSize({width:1280,height:800});
await app.done();
