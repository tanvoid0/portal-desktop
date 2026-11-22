// Repro: Tasks rail shows "Project #N" placeholders while projects load / for dangling project links.
import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('tasks');
const p = app.page;
await dismissVault(p);
const sample = () => p.evaluate(() => [...document.querySelectorAll('span[title^="Project #"]')].map(e => e.title));
console.log('settled placeholders:', await sample());
for (let run = 0; run < 3; run++) {
  await p.goto(new URL('/tasks', p.url()).href);
  const seen = new Set(); const t0 = Date.now();
  while (Date.now() - t0 < 4000) { (await sample()).forEach(x => seen.add(x)); await p.waitForTimeout(30); }
  console.log('reload', run, 'placeholders seen:', [...seen]);
  await dismissVault(p, 500);
}
await app.done();
