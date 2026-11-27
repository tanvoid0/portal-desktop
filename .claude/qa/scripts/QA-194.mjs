// Repro: a long project name pushes "Resolve with AI" out of the Sync all sheet. Also checks the
// two-drives toast and the filter toggle. Stubbed report, no sync runs.
import { fakeSync, rows, open } from './sync-all-stub.mjs';

const app = await open('projects');
const { page } = app;
try {
  await page.setViewportSize({ width: 1024, height: 600 });
  await fakeSync(page, rows());
  await page.waitForTimeout(800);
  const list = await page
    .getByRole('dialog')
    .locator('ul')
    .evaluate((u) => [u.scrollWidth, u.clientWidth]);
  console.log('list scrollWidth/clientWidth', list, list[0] > list[1] ? 'OVERFLOW' : 'ok');
  console.log(await app.shot('QA-194-before-syncall-long-name'));

  const attention = page.getByRole('radio', { name: 'Needs attention' }).or(page.getByRole('button', { name: 'Needs attention' }));
  await attention.click();
  await page.waitForTimeout(300);
  console.log(await app.shot('sweep-projects-syncall-filter-clicked-twice'));

  await fakeSync(page, [rows()[0], rows('E:\\QA fixtures')[2]]);
  await page.getByRole('button', { name: /Resolve all with AI/ }).click();
  await page.waitForTimeout(500);
  console.log('url', page.url());
  console.log(await app.shot('sweep-projects-syncall-two-drives-toast'));
} finally {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('http://tauri.localhost/');
  await app.done();
}
