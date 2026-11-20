// Clones the real CapabilityChecklist grid into fixed-width wrappers (no pending request exists on dev data).
import { connect } from './cdp.mjs';
const app = await connect();
await app.nav('/devices', 2000);
await app.metrics(1280, 800);
const r = await app.eval(() => {
  const g = [...document.querySelectorAll('div')].find(d => getComputedStyle(d).display==='grid' && d.querySelector('button[role=switch]') && d.children.length>3);
  const res = [];
  const holder = document.createElement('div');
  holder.id='__qa122'; holder.style.cssText='position:fixed;top:0;left:0;z-index:99999;background:var(--background);padding:8px;display:flex;gap:12px;align-items:flex-start;overflow:auto;max-width:100vw';
  document.body.appendChild(holder);
  for (const w of [138, 290, 330, 456]) {
    const box = document.createElement('div'); box.style.cssText=`width:${w}px;border:1px solid red;flex:none`;
    const c = g.cloneNode(true); box.appendChild(c); holder.appendChild(box);
    const gr = c.getBoundingClientRect();
    const bad = [...c.children].filter(x => { const b=x.getBoundingClientRect(); return b.right>gr.right+1 || x.scrollWidth>x.clientWidth+1; }).length;
    res.push({w, cols:getComputedStyle(c).gridTemplateColumns.split(' ').length, cells:c.children.length, overflow:bad});
  }
  return res;
});
console.log(JSON.stringify(r));
console.log(await app.shot('QA-122-after-narrow-clone-widths'));
await app.eval(()=>document.getElementById('__qa122')?.remove());
await app.resetMetrics(); await app.close(); process.exit(0);
