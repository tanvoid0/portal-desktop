import { open } from './app.mjs';
const routes = process.argv.slice(2);
const app = await open('');
const p = app.page;
await p.waitForTimeout(1000);
await p.getByRole('button',{name:'Not now'}).click({timeout:800}).catch(()=>{});
for (const r of routes) {
  await p.goto(new URL('/'+r, p.url()).href);
  await p.waitForTimeout(2000);
  await p.getByRole('button',{name:'Not now'}).click({timeout:500}).catch(()=>{});
  await p.waitForTimeout(400);
  await app.shot('sweep-'+r.replace(/\//g,'-')+'-1280');
  console.log('##',r,'|',(await p.locator('main').innerText().catch(()=>'')).replace(/\n+/g,' | ').slice(0,350));
}
await app.done();
