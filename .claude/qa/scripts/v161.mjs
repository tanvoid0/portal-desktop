import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('coder');
const p = app.page;
await dismissVault(p, 6000);
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({ width: w, height: h }); await p.waitForTimeout(1200);
  console.log(await app.shot(`QA-161-after-coder-clear-${w}x${h}`));
}
await p.setViewportSize({ width: 1280, height: 800 });
await app.done();
