// Repro: credential card icon buttons (reveal, edit, delete, copy) have no accessible name. Clicks nothing.
import { open } from './app.mjs';
const app = await open('credentials');
await app.page.waitForTimeout(1500);
const n = await app.page.locator('main button').evaluateAll((b) => b.filter((x) => !x.getAttribute('aria-label') && !x.innerText.trim() && !x.title).length);
console.log('unnamed buttons:', n, '(expected 0)');
await app.done();
