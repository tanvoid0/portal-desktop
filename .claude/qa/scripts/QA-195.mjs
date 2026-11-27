// Repro: is the "no parent folder" error toast visible while the Sync all sheet is open? Stubbed, no sync.
import { fakeSync, rows, open } from './sync-all-stub.mjs';
const app = await open('projects');
const { page } = app;
try {
  await fakeSync(page, [rows()[0], rows('E:/QA fixtures')[2]]);
  await page.waitForTimeout(800);
  await page.getByRole('button', { name: /Resolve all with AI/ }).click();
  await page.waitForTimeout(1500);
  const t = page.getByText('These projects share no parent folder', { exact: false });
  console.log('toast nodes', await t.count());
  if (await t.count())
    console.log(await t.first().evaluate((el) => {
      const r = el.getBoundingClientRect();
      const top = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
      return { rect: [r.x, r.y, r.width, r.height].map(Math.round), onTop: el.contains(top) || top?.contains(el), covering: String(top?.className).slice(0, 80) };
    }));
  console.log(await app.shot('QA-195-before-syncall-no-parent-toast'));
} finally {
  await page.goto('http://tauri.localhost/');
  await app.done();
}
