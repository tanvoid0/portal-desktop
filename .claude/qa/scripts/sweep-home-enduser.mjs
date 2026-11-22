import { open } from './app.mjs';
const app = await open('');
const p = app.page;
await p.getByRole('button',{name:'Not now'}).click({timeout:1500}).catch(()=>{});
for (const t of [500,3000,6000]) { await p.waitForTimeout(t);
 console.log(t,(await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,200), '| sidebar', (await p.locator('nav, aside').first().innerText()).replace(/\n+/g,' ').slice(0,150)); }
await app.shot('sweep-home-settled-1280');
await app.done();
