import { open } from './app.mjs';
const app = await open('tasks/68/edit'); const p = app.page;
const get = ()=>p.evaluate(async()=>{const t=await window.__TAURI_INTERNALS__.invoke('get_task',{id:68});return [t.status,t.completed_at]});
console.log('before', await get());
await p.fill('#title','QA Beta edited'); await p.getByRole('button',{name:'Update Task'}).click(); await p.waitForTimeout(1500);
console.log('after title-only edit', await get(), p.url());
await p.goto(new URL('/tasks/68/edit',p.url()).href); await p.waitForTimeout(1000);
await p.fill('#title','QA Beta'); await p.getByRole('button',{name:'Update Task'}).click(); await p.waitForTimeout(1500);
await app.done();
