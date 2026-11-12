// usage: node qa-pipe-tpl.mjs <projectId> <template name> <shot>
import { open } from './app.mjs';
const [id, tpl, shot] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines/new`); const p = app.page;
await p.waitForTimeout(3000);
await p.locator('button', { hasText: 'All Templates' }).click(); await p.waitForTimeout(1000);
await p.getByText(tpl, { exact: true }).first().click(); await p.waitForTimeout(3000);
console.log(p.url());
console.log((await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,3000));
console.log(await app.shot(shot));
await app.done();
