import { open } from './app.mjs';
const app = await open(process.argv[2]||'tasks?project=all&view=board'); const p = app.page;
await p.setViewportSize({width:1024,height:700}); await p.waitForTimeout(800);
const m = await p.evaluate(()=>{
  const root=document.querySelector('div.flex.gap-4.overflow-x-auto'); const par=root.parentElement;
  const rr=root.getBoundingClientRect(), pr=par.getBoundingClientRect();
  return {rootH:Math.round(rr.height), rootTop:Math.round(rr.top), rootBottom:Math.round(rr.bottom), parentH:par.clientHeight, parentSH:par.scrollHeight, parentBottom:Math.round(pr.bottom), vh:innerHeight, rootOY:getComputedStyle(root).overflowY, hscrollbarVisibleWithoutScrolling: rr.bottom<=pr.bottom};
});
console.log(JSON.stringify(m));
await app.shot('sweep-tasks-hscroll-1024x700-top');
// wheel over board: vertical wheel -> does board move horizontally?
const box = await p.locator('div.flex.gap-4.overflow-x-auto').boundingBox();
await p.mouse.move(box.x+100, box.y+100);
const before = await p.evaluate(()=>document.querySelector('div.flex.gap-4.overflow-x-auto').scrollLeft);
await p.mouse.wheel(0, 300); await p.waitForTimeout(300);
const afterV = await p.evaluate(()=>({sl:document.querySelector('div.flex.gap-4.overflow-x-auto').scrollLeft, pst:document.querySelector('div.flex.gap-4.overflow-x-auto').parentElement.scrollTop}));
await p.mouse.wheel(300, 0); await p.waitForTimeout(300);
const afterH = await p.evaluate(()=>document.querySelector('div.flex.gap-4.overflow-x-auto').scrollLeft);
console.log('scrollLeft before',before,'after vertical wheel',JSON.stringify(afterV),'after horizontal wheel',afterH);
await p.evaluate(()=>{const r=document.querySelector('div.flex.gap-4.overflow-x-auto'); r.parentElement.scrollTop=99999;}); await p.waitForTimeout(300);
await app.shot('sweep-tasks-hscroll-1024x700-bottom');
await p.setViewportSize({width:1280,height:800});
await app.done();
