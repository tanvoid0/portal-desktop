import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({width:w,height:h}); await p.waitForTimeout(400);
  await p.keyboard.press('Control+f'); await p.waitForTimeout(500);
  const m = await p.evaluate(()=>{const d=document.querySelector('[role=dialog]');const r=d.getBoundingClientRect();const b=[...d.querySelectorAll('button')].find(b=>/Apply/.test(b.textContent));const br=b.getBoundingClientRect();const sc=[...d.querySelectorAll('*')].filter(e=>e.scrollHeight>e.clientHeight+2&&/auto|scroll/.test(getComputedStyle(e).overflowY)).map(e=>e.className.toString().slice(0,40));return {dlg:[Math.round(r.top),Math.round(r.bottom)],apply:[Math.round(br.top),Math.round(br.bottom)],ovf:getComputedStyle(d).overflowY,dh:d.scrollHeight,dc:d.clientHeight,sc}});
  console.log(w,h,JSON.stringify(m)); await app.shot(`sweep-tasks-v2-filter-${w}x${h}`);
  await p.keyboard.press('Escape'); await p.waitForTimeout(300);
}
await p.setViewportSize({width:1280,height:800});
await app.done();
