// Pipeline run page: when the execution cannot be loaded it stays on "Loading execution..." forever.
import { open } from './app.mjs';
const app = await open('projects/1/pipelines/run/999');
await app.page.waitForTimeout(5000);
const t = await app.page.locator('main').innerText();
console.log(t.includes('Loading execution') ? 'BUG: still "Loading execution..." after 6s' : 'ok: ' + t.slice(0, 100));
await app.shot('QA-003-run-loading');
await app.done();
