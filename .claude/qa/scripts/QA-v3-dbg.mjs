import { open } from './app.mjs';
const app = await open('tasks?project=all&view=archive'); const p = app.page;
p.on('request', r=>{ if(/ipc|invoke/.test(r.url())) console.log('REQ', r.url().slice(0,80)); });
await p.reload(); await p.waitForTimeout(2000);
console.log(await p.evaluate(()=>Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'invoke')));
await app.done();
