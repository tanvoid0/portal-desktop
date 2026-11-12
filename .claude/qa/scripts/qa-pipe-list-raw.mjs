import { open } from './app.mjs';
const app = await open('projects/14'); const p = app.page;
const r = await p.evaluate(async () => { const l = await window.__TAURI_INTERNALS__.invoke('get_pipelines', { projectId: 14 }); return l.map(x => ({ id: x.id, name: x.name, steps: x.steps?.length })); });
console.log(JSON.stringify(r));
await app.done();
