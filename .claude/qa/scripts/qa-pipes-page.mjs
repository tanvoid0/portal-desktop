import { open } from './app.mjs';
const app = await open(`pipelines`); const p = app.page;
await p.waitForTimeout(4000);
console.log((await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,2500));
console.log(await app.shot('sweep-pipelines-page-release'));
await app.done();
