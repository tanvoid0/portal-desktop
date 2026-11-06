import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
console.log('mode', app.mode, p.url());
for (const [w,h] of [[1280,800],[1024,700],[900,700],[1024,600]]) {
  await p.setViewportSize({width:w,height:h}); await p.waitForTimeout(700);
  const m = await p.evaluate(()=>{
    const root=document.querySelector('div.flex.h-full.min-h-0.overflow-x-auto');
    if(!root) return {err:'no board root'};
    const r=root.getBoundingClientRect();
    const cols=[...root.children].map(c=>{const sc=c.querySelector('.overflow-y-auto');return sc?{sh:sc.scrollHeight,ch:sc.clientHeight}:null});
    const hdr=root.children[0]?.firstElementChild?.getBoundingClientRect();
    return {vh:innerHeight, docH:document.documentElement.scrollHeight, rootSW:root.scrollWidth, rootCW:root.clientWidth, rootCH:root.clientHeight, rootBottom:Math.round(r.bottom), hbarVisible: r.bottom<=innerHeight+1, cols, hdrTop:Math.round(hdr?.top)};
  });
  console.log(w+'x'+h, JSON.stringify(m));
}
await p.setViewportSize({width:1024,height:700}); await p.waitForTimeout(500);
// drag scrollbar equivalent: scroll to max, check last column fully visible
const res = await p.evaluate(async()=>{const root=document.querySelector('div.flex.h-full.min-h-0.overflow-x-auto');root.scrollLeft=99999;await new Promise(r=>setTimeout(r,200));const last=root.lastElementChild.getBoundingClientRect(),rr=root.getBoundingClientRect();return {scrollLeft:root.scrollLeft,lastRight:Math.round(last.right),rootRight:Math.round(rr.right),lastLeft:Math.round(last.left)}});
console.log('scrollMax', JSON.stringify(res));
await app.shot('sweep-tasks-hscroll-fixed-1024x700');
// column vertical scroll independent
const col = await p.evaluate(async()=>{const root=document.querySelector('div.flex.h-full.min-h-0.overflow-x-auto');root.scrollLeft=0;const c=root.children[0];const sc=c.querySelector('.overflow-y-auto');const hd=c.firstElementChild;const before=hd.getBoundingClientRect().top;sc.scrollTop=300;await new Promise(r=>setTimeout(r,200));return {scrollTop:sc.scrollTop,hdrMoved:hd.getBoundingClientRect().top-before,footer:c.lastElementChild===sc?'none':c.lastElementChild.textContent.trim().slice(0,40), footerBottom:Math.round(c.lastElementChild.getBoundingClientRect().bottom), pageScroll:document.scrollingElement.scrollTop}});
console.log('colscroll', JSON.stringify(col));
await app.shot('sweep-tasks-col-scrolled');
console.log('breadcrumb:', (await p.locator('nav[aria-label*="readcrumb" i]').first().innerText().catch(()=> 'n/a')).replace(/\n/g,' | '));
await app.done();
