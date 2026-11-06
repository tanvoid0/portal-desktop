import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
const get = (id)=>p.evaluate(async id=>{const t=await window.__TAURI_INTERNALS__.invoke('get_task',{id});return t&&{s:t.status,c:t.completed_at??t.completedAt,u:t.updated_at??t.updatedAt}},id);
const col = (n)=>p.getByRole('region',{name:`Drop zone for ${n} tasks`});
const card=(t)=>p.getByRole('button',{name:`Select task: ${t}`});
console.log('start', JSON.stringify(await get(68)));
for (const c of ['In Progress','Done','Backlog','Done','In Progress','Backlog']) {
  await card('QA Beta').dragTo(col(c)); await p.waitForTimeout(1000);
  console.log('->',c, JSON.stringify(await get(68)), 'inCol', await col(c).getByText('QA Beta').count());
}
await app.shot('sweep-tasks-v2-drag-done');
await card('QA Beta').dragTo(col('Done')); await p.waitForTimeout(1000);
await p.reload(); await p.waitForTimeout(1500);
console.log('after reload Done has Beta', await col('Done').getByText('QA Beta').count(), JSON.stringify(await get(68)));
await p.goto(new URL('/tasks/68',p.url()).href); await p.waitForTimeout(1200);
console.log('detail:', (await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,300));
await app.done();
