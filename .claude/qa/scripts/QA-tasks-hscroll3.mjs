import { open } from './app.mjs';
for (const [route,label] of [['tasks?project=all&view=list','list'],['tasks?project=all&view=archive','archive'],['github/issues','github']]) {
  const app = await open(route); const p = app.page;
  await p.setViewportSize({width:760,height:700}); await p.waitForTimeout(800);
  const m = await p.evaluate(()=>{
    const root=document.querySelector('div.flex.gap-4.overflow-x-auto');
    const main=document.querySelector('main'); const out={board:null};
    if(root){const r=root.getBoundingClientRect(); out.board={sw:root.scrollWidth,cw:root.clientWidth,h:Math.round(r.height),bottom:Math.round(r.bottom)};}
    // widest descendant right edge vs its clipping ancestor
    let worst=null; document.querySelectorAll('main *').forEach(e=>{const r=e.getBoundingClientRect(); if(r.right>innerWidth+1&&r.width>0&&!worst) worst=[e.tagName,(e.className||'').toString().slice(0,50),Math.round(r.right)]});
    out.firstBeyondViewport=worst; out.docSW=document.documentElement.scrollWidth; return out;});
  console.log(label, JSON.stringify(m));
  await app.shot(`sweep-tasks-hscroll-${label}-760`);
  await p.setViewportSize({width:1280,height:800}); await app.done();
}
