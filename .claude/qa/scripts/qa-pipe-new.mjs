import { open } from './app.mjs';
const id = process.argv[2];
const app = await open(`projects/${id}/pipelines/new`); const p = app.page;
await p.waitForTimeout(3000);
console.log(p.url());
console.log((await p.locator('body').innerText()).slice(0, 3000));
console.log(await app.shot(`sweep-pipelines-new-${id}-release`));
await app.done();
