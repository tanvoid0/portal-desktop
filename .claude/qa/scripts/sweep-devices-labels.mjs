import { open } from './app.mjs';
const app = await open('devices');
const p = app.page;
await p.getByRole('button',{name:'Not now'}).click().catch(()=>{});
const r = await p.locator('main button').evaluateAll(bs=>bs.map(b=>[b.getAttribute('aria-label'),b.title,b.innerText.trim()].join('|')));
console.log(r.join('\n'));
await app.done();
