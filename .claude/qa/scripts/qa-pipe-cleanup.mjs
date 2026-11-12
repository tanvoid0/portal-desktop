import { open } from './app.mjs';
const app = await open('projects/14'); const p = app.page;
const r = await p.evaluate(async () => { const l = await window.__TAURI_INTERNALS__.invoke('get_pipelines', { projectId: 14 }); for (const x of l.filter(x => x.name.startsWith('QA '))) await window.__TAURI_INTERNALS__.invoke('delete_pipeline', { pipelineId: x.id }); return (await window.__TAURI_INTERNALS__.invoke('get_pipelines', { projectId: 14 })).length; });
console.log('remaining pipelines for 14:', r);
await app.done();
