import { open } from './app.mjs';
const app = await open('projects/14'); const p = app.page;
console.log(p.url());
console.log(JSON.stringify(await p.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('get_pipelines', { projectId: 14 })).map(x=>[x.id,x.name]))));
console.log(JSON.stringify((await p.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('get_all_projects')).length))));
await app.done();
