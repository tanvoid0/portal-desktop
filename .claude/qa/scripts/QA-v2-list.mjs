import { open } from './app.mjs';
const app = await open('tasks'); const p = app.page;
const r = await p.evaluate(async()=>{const a=await window.__TAURI_INTERNALS__.invoke('get_main_tasks');return a.filter(t=>/^QA/.test(t.title)).map(t=>[t.id,t.title.slice(0,30),t.status,t.completed_at])});
console.log(JSON.stringify(r)); await app.done();
