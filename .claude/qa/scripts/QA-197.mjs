// QA-197: the two chart series (and their legend dots) are the same blue.
import { open } from './app.mjs';
const app = await open('system');
const { page } = app;
const wasDark = await page.evaluate(() => document.documentElement.classList.contains('dark'));
await page.waitForTimeout(8000);
for (const dark of [false, true]) {
  await page.evaluate((d) => document.documentElement.classList.toggle('dark', d), dark);
  await page.waitForTimeout(300);
  const c = await page.evaluate(() => [...document.querySelectorAll('polyline')].slice(0, 2).map((p) => getComputedStyle(p).stroke));
  console.log(dark ? 'dark ' : 'light', 'series A', c[0], '| series B', c[1]);
}
console.log(await app.shot('QA-197-before-chart-series'));
await page.evaluate((d) => document.documentElement.classList.toggle('dark', d), wasDark);
await app.done();
