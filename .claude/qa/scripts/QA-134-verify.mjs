import { connect } from './cdp.mjs';
const app = await connect();
const K='portal-coder-last-workspace';
const stale='/media/tan/Workspace/production/portal/portal_desktop';
await app.nav('/', 1000);
console.log('before', await app.eval((k)=>localStorage.getItem(k),K));
await app.eval((k,v)=>localStorage.setItem(k,v),K,stale);
await app.eval(()=>location.href='/ai/coder'); await app.wait(5000);
console.log(await app.eval(()=>location.pathname));
console.log('after-stale', await app.eval((k)=>localStorage.getItem(k),K));
console.log('body has /media:', await app.eval(()=>document.body.innerText.includes('/media/tan')));
console.log(await app.shot('QA-134-after-stale-dropped'));
// real project path stays
const real = await app.eval((k)=>localStorage.getItem(k),K);
if (real) {
  await app.eval((k,v)=>localStorage.setItem(k,v),K,real);
  await app.eval(()=>location.reload()); await app.wait(5000);
  console.log('real kept:', await app.eval((k)=>localStorage.getItem(k),K)===real);
}
await app.close(); process.exit(0);
