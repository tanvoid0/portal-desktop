import { open } from './app.mjs';
const app = await open('tasks'); const p = app.page;
for (const id of [79,80,81,82]) console.log(id, await p.evaluate(async id=>{const t=await window.__TAURI_INTERNALS__.invoke('get_task',{id}); if(!t||!/^QA /.test(t.title)) return 'skip'; await window.__TAURI_INTERNALS__.invoke('delete_task',{id}); return 'deleted '+t.title.slice(0,12)},id));
await app.done();
