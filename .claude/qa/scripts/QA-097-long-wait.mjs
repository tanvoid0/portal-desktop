import { open } from './app.mjs';

const app = await open('sdk');
for (const t of [3, 8, 15, 25, 40, 60]) {
  await app.page.waitForTimeout(t === 3 ? 3000 : (t - [3, 8, 15, 25, 40, 60][[3, 8, 15, 25, 40, 60].indexOf(t) - 1]) * 1000);
  const state = await app.page.evaluate(() => ({
    stillLoading: document.body.innerText.includes('Loading SDK data'),
    hasQuickNav: document.body.innerText.includes('Quick Navigation'),
  }));
  console.log(`t=${t}s:`, JSON.stringify(state));
}
await app.shot('QA-097-after-60s-wait');
await app.done();
