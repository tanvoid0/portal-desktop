import { open } from './app.mjs';
const app = await open('ai/coder'); const p = app.page; await p.waitForTimeout(3000);
const t = await p.evaluate(async () => { try { const r = await window.__TAURI_INTERNALS__.invoke('coder_list_threads', {}); return JSON.stringify(r).slice(0,100) } catch(e){ return 'ERR '+e } });
console.log(t);
const body = await p.locator('body').innerText(); console.log(body.slice(0,1200));
console.log(await app.shot('sweep-doctor-coder-threads-after-reload-dev')); await app.done();
