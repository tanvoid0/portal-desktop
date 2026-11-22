// Repro: project header path chip prints literal "${parts.slice(-3).join('')}"
import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('projects/233');
await dismissVault(app.page);
const txt = await app.page.evaluate(() => document.body.innerText.match(/.*\$\{parts.*/)?.[0] ?? null);
console.log(txt ? 'BUG: ' + txt : 'ok: no literal template in path chip');
console.log(await app.shot('QA-171-before-path-chip-233'));
await app.done();
