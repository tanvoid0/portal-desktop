import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('coder');
const p = app.page;
await dismissVault(p, 4000);
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({ width: w, height: h });
  for (const [id,r] of [['161','coder'],['161','ai/chat']]) {
    await p.goto(new URL('/'+r, p.url()).href); await dismissVault(p,1500); await p.waitForTimeout(2500);
    console.log(id, r, w, await app.shot(`QA-${id}-after-${r.replace('/','-')}-${w}x${h}`));
  }
}
await p.setViewportSize({ width: 1280, height: 800 });
for (const [id,r] of [['162','credentials'],['163','devices'],['164','sdk'],['165','github'],['166','ai/chat'],['167','cloud/workloads']]) {
  await p.goto(new URL('/'+r, p.url()).href); await dismissVault(p,1500); await p.waitForTimeout(2500);
  console.log(id, await app.shot(`QA-${id}-after-${r.replace('/','-')}`));
}
await app.done();
