import { connect } from './cdp.mjs';
const app = await connect();
await app.nav('/database', 3500); await app.eval(()=>document.querySelector('[role=dialog] button[aria-label=Close], [role=dialog] button:has(svg.lucide-x)')?.click()); await app.wait(8000);
const st = () => app.eval(()=>{
  const b=document.querySelector('button[aria-label$=" tables"][aria-expanded]');
  const items=document.querySelectorAll('[id$="-tables"] > *').length;
  const f=document.querySelector('input[aria-label^="Filter "]');
  return {chev: b&&b.getAttribute('aria-label'), expanded:b&&b.getAttribute('aria-expanded'), rows:items, filter:!!f, status:[...document.querySelectorAll('p')].map(p=>p.innerText).filter(t=>/ of \d+ tables|No tables/.test(t))};
});
console.log('initial', JSON.stringify(await st()));
console.log(await app.shot('UX-022-after-collapsed-initial'));
await app.eval(()=>document.querySelector('button[aria-label$=" tables"][aria-expanded]').click()); await app.wait(600);
console.log('expanded', JSON.stringify(await st()));
console.log(await app.shot('UX-022-after-expanded'));
// filter
await app.eval(()=>{const i=document.querySelector('input[aria-label^="Filter "]'); const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set; set.call(i,'agent_'); i.dispatchEvent(new Event('input',{bubbles:true}));}); await app.wait(500);
console.log('filtered agent_', JSON.stringify(await st()));
console.log(await app.shot('UX-022-after-filter-agent'));
await app.eval(()=>{const i=document.querySelector('input[aria-label^="Filter "]'); const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set; set.call(i,'zzzqq'); i.dispatchEvent(new Event('input',{bubbles:true}));}); await app.wait(500);
console.log('nomatch', JSON.stringify(await st()));
// collapse -> clears filter
await app.eval(()=>document.querySelector('button[aria-label$=" tables"][aria-expanded]').click()); await app.wait(500);
console.log('collapsed again', JSON.stringify(await st()), 'filterval', await app.eval(()=>document.querySelector('input[aria-label^="Filter "]')?.value));
// click a table works
await app.eval(()=>document.querySelector('button[aria-label$=" tables"][aria-expanded]').click()); await app.wait(500);
console.log('errors', app.errors.slice(0,3));
await app.close(); process.exit(0);
