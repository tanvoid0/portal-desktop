import { open } from './app.mjs';
const app = await open('tasks/create'); const p = app.page;
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({width:w,height:h});
  for (const [route,btn] of [['/tasks/create','Create Task'],['/tasks/67',null],['/tasks/67/edit',/Save|Update/],['/tasks/generate',/Generate Tasks/]]) {
    await p.goto(new URL(route,p.url()).href); await p.waitForTimeout(1200);
    const r = await p.evaluate(async()=>{
      const sc=[...document.querySelectorAll('body *')].filter(e=>/(auto|scroll)/.test(getComputedStyle(e).overflowY)&&e.scrollHeight>e.clientHeight+2).sort((a,b)=>b.scrollHeight-a.scrollHeight)[0];
      if(!sc) return {noScroller:true,docH:document.documentElement.scrollHeight,vh:innerHeight};
      sc.scrollTop=1e6; await new Promise(r=>setTimeout(r,250));
      return {c:(sc.className||'').toString().slice(0,50),sh:sc.scrollHeight,ch:sc.clientHeight,st:Math.round(sc.scrollTop),vh:innerHeight};
    });
    let btnInfo='';
    if(btn){ const b=p.getByRole('button',{name:btn}).last(); const bb=await b.boundingBox().catch(()=>null); btnInfo=bb?`btnBottom=${Math.round(bb.y+bb.height)} inView=${bb.y+bb.height<=h}`:'btn not found'; }
    console.log(`${w}x${h} ${route}`, JSON.stringify(r), btnInfo);
    if(w===1024) await app.shot('sweep-tasks-'+route.slice(7).replace(/\//g,'-')+'-bottom-1024x600');
  }
}
await app.done();
