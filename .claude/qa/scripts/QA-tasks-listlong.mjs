import { open } from './app.mjs';
const app = await open('tasks?project=none&view=list'); const p = app.page;
const m = await p.evaluate(()=>{const o=[];document.querySelectorAll('main *').forEach(e=>{if(e.textContent?.startsWith('QA Nospace')&&e.children.length===0){const r=e.getBoundingClientRect();o.push({w:Math.round(r.width),right:Math.round(r.right),sw:e.scrollWidth,cw:e.clientWidth})}});return {o,vw:innerWidth}});
console.log(JSON.stringify(m)); await app.shot('QA-077-before-list-nospace-title');
await app.done();
