import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
for (const id of [84,85,86,87]) console.log(id, await p.evaluate(async(id)=>{try{const t=await window.__TAURI_INTERNALS__.invoke('get_task',{id}); if(!/^QA Vfy /.test(t.title)) return 'SKIP '+t.title; await window.__TAURI_INTERNALS__.invoke('delete_task',{id}); return 'deleted '+t.title}catch(e){return 'ERR '+JSON.stringify(e)}},id));
await p.reload(); await p.waitForTimeout(1500);
console.log('QA left:', (await p.locator('main').innerText()).match(/QA [^\n]*/g));
await app.done();
