import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
for (const [w,h] of [[1280,800],[1024,700],[900,700],[1024,600]]) {
  await p.setViewportSize({width:w,height:h}); await p.waitForTimeout(700);
  const m = await p.evaluate(()=>{
    const root=[...document.querySelectorAll('div')].find(e=>/overflow-x-auto/.test(e.className)&&/flex/.test(e.className)&&e.scrollWidth>0&&e.querySelectorAll('[class*=overflow-y-auto]').length>0);
    if(!root) return {err:'no root'};
    const r=root.getBoundingClientRect();
    const cols=[...root.children].map(c=>{const cr=c.getBoundingClientRect();const sc=c.querySelector('[class*=overflow-y-auto]');return {w:Math.round(cr.width),h:Math.round(cr.height),scH:sc?sc.scrollHeight:0,scC:sc?sc.clientHeight:0}});
    return {sw:root.scrollWidth,cw:root.clientWidth,ch:root.clientHeight,bottom:Math.round(r.bottom),vh:innerHeight,docH:document.documentElement.scrollHeight,bodyH:document.body.scrollHeight,cols, bar: root.offsetHeight-root.clientHeight};
  });
  console.log(w,h,JSON.stringify(m));
  await p.mouse.move(300,300);
  await app.shot(`QA-076-after-${w}x${h}`);
}
await app.done();
