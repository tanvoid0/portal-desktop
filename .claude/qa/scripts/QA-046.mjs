// Repro: icon-only Stop/Start/Remove buttons on container rows have no accessible name. Reads only; clicks nothing.
import { open } from './app.mjs';
const app = await open('deployments');
const p = app.page;
await p.waitForTimeout(5000);
const n = await p.locator('main button').evaluateAll((b) => b.filter((x) => !x.getAttribute('aria-label') && !x.innerText.trim() && !x.title).length);
console.log('buttons with no accessible name:', n, '(expected 0)');
await app.done();
