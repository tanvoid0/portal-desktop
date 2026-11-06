import { open } from './app.mjs';
const app = await open('tasks?project=all&view=list'); const p = app.page;
await p.setViewportSize({width:1024,height:600}); await p.waitForTimeout(700);
const r = await p.getByText(/^QA Nospace/).first().evaluate(e=>{const out=[];for(let x=e,i=0;x&&i<9;x=x.parentElement,i++){const cs=getComputedStyle(x);out.push([x.tagName,Math.round(x.getBoundingClientRect().width),cs.display,cs.minWidth,x.className.toString().slice(0,90)])}return out});
r.forEach(x=>console.log(JSON.stringify(x)));
await app.done();
