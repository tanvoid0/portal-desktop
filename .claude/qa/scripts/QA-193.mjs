// Repro: "Resolve all with AI" must open the coder in the repos' shared parent folder, even when
// that folder is not itself a Portal project. Stubbed report (no sync runs), real folder paths,
// the prompt is never sent.
import { fakeSync, rows, open } from './sync-all-stub.mjs';

const app = await open('projects');
const { page } = app;
const ls = await page.evaluate(() => ({ ...localStorage }));
const paths = (
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('get_all_projects'))
).map((p) => p.path);
const parent = (p) => p.split(/[\\/]/).slice(0, -1).join('\\');
const norm = (p) => p.replace(/\//g, '\\').toLowerCase();
const known = new Set(paths.map(norm));
const pair = paths
  .map((a) => [a, paths.find((b) => b !== a && norm(parent(b)) === norm(parent(a)))])
  .find(([a, b]) => b && parent(a).split('\\').length > 2 && !known.has(norm(parent(a))));
console.log('stuck repos', pair, '\nexpected workspace', parent(pair[0]));

const fx = rows()[0];
try {
  await fakeSync(
    page,
    pair.map((path, i) => ({ ...fx, id: 990001 + i, name: `QA repo ${i + 1}`, path }))
  );
  await page.getByRole('button', { name: /Resolve all with AI/ }).click();
  await page.waitForTimeout(4000);
  console.log(
    'actual workspace',
    await page.evaluate(() => localStorage.getItem('portal-coder-last-workspace'))
  );
  console.log(await app.shot('QA-193-before-coder-wrong-workspace'));
  await page.getByPlaceholder(/Ask the coder/i).fill('');
} finally {
  await page.evaluate((s) => {
    localStorage.clear();
    for (const [k, v] of Object.entries(s)) localStorage.setItem(k, v);
  }, ls);
  await page.goto('http://tauri.localhost/');
  await app.done();
}
