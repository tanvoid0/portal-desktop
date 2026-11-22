import { open } from './app.mjs';
const app = await open('devices');
const p = app.page;
await p.getByRole('button',{name:'Not now'}).click().catch(()=>{});
await p.waitForTimeout(600);
await app.shot('sweep-devices-after-dismiss-dark-1280');
console.log((await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,2500));
// narrow
await p.setViewportSize({width:1024,height:600});
await p.waitForTimeout(500);
await app.shot('sweep-devices-dark-1024x600');
// open details of a device: click pencil
await p.locator('button:has(svg)').filter({hasText:''}).nth(0);
await app.done();
