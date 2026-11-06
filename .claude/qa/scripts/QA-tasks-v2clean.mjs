import { open } from './app.mjs';
const app = await open('tasks'); const p = app.page;
for (const id of [67,68,69,70,71,72]) console.log(id, await p.evaluate(async id=>{try{const t=await window.__TAURI_INTERNALS__.invoke('get_task',{id});if(!t||!String(t.title).startsWith('QA '))return 'skip '+(t&&t.title);await window.__TAURI_INTERNALS__.invoke('delete_task',{id});return 'deleted '+t.title}catch(e){return 'gone'}},id));
await app.done();
