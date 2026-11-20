import { connect } from './cdp.mjs';
const app = await connect();
const snap = () => app.eval(()=>({path:location.pathname, txt:document.querySelector('main')?.innerText.replace(/\s+/g,' ').slice(0,200)}));
// cold: hard reload on /projects, sample early
await app.eval(()=>{location.href='/projects'});
const t0=Date.now();
for (const ms of [150,400,800,1500,3000,6000]) { await app.wait(ms - (Date.now()-t0) > 0 ? ms-(Date.now()-t0) : 0); try{ console.log(ms, JSON.stringify(await snap())); }catch(e){ console.log(ms,'eval err'); } }
console.log(await app.shot('QA-133b-live-projects-list'));
// pick first project id link
const href = await app.eval(()=>[...document.querySelectorAll('a[href^="/projects/"]')].map(a=>a.getAttribute('href')).filter(h=>/^\/projects\/\d+$/.test(h))[0]);
console.log('detail href', href);
await app.nav(href, 100);
const t1=Date.now();
for (const ms of [300,1000,2500,5000,10000]) { await app.wait(ms-(Date.now()-t1)); console.log(ms, JSON.stringify(await snap())); }
console.log(await app.shot('QA-133b-live-project-detail-id'));
// cold detail
await app.eval((h)=>{location.href=h}, href);
for (const ms of [500,2000,5000]) { await app.wait(ms===500?500:ms===2000?1500:3000); console.log('cold',ms, JSON.stringify(await snap())); }
console.log(app.errors.slice(0,3));
await app.close(); process.exit(0);
