import { connect } from './cdp.mjs';
const app = await connect();
console.log(await app.nav('/devices', 2500));
for (const w of [1280, 1024]) {
  await app.metrics(w, 800);
  const r = await app.eval(() => {
    const out = { pending: /Pending requests/.test(document.body.innerText) };
    out.switches = document.querySelectorAll('button[role=switch]').length;
    const grids = [...document.querySelectorAll('div')].filter(d => getComputedStyle(d).display==='grid' && d.querySelector('button[role=switch]') && d.children.length>3);
    out.grids = grids.map(g => { const gr=g.getBoundingClientRect(); const bad=[...g.children].filter(c=>c.getBoundingClientRect().right>gr.right+1||c.scrollWidth>c.clientWidth+1).length; return {w:Math.round(gr.width),cols:getComputedStyle(g).gridTemplateColumns.split(' ').length,cells:g.children.length,overflow:bad}; });
    return out;
  });
  console.log(w, JSON.stringify(r));
}
await app.resetMetrics();
console.log(await app.shot('QA-122-after-devices-grid'));
console.log(app.errors.slice(0,3));
await app.close();
process.exit(0);
