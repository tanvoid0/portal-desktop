import { open } from './app.mjs';
const app = await open('tasks?project=all&view=archive'); const p = app.page;
// client-side response rewrite only (no data written); route is dropped when the script ends.
const dates={79:'2026-10-02T10:00:00Z',80:'2026-10-20T10:00:00Z',81:'2026-09-15T10:00:00Z',82:'2026-10-28T10:00:00Z'};
await p.route('**/ipc.localhost/get_tasks', async route=>{
  const res=await route.fetch(); let j; try{j=await res.json()}catch{return route.fulfill({response:res})}
  const fix=t=>{if(t&&dates[t.id]){t.status='completed';t.completed_at=dates[t.id];}return t};
  const out=Array.isArray(j)?j.map(fix):j; console.log('patched', Array.isArray(j)?j.length:typeof j);
  await route.fulfill({response:res, json:out});
});
await p.reload(); await p.waitForTimeout(2500);
console.log((await p.locator('main').innerText()).replace(/\n+/g,' | ').match(/Filters.*/)?.[0].slice(0,900));
await app.shot('sweep-tasks-archive-month-grouping');
await p.unroute('**/ipc.localhost/get_tasks');
await p.reload(); await p.waitForTimeout(1500);
await app.done();
