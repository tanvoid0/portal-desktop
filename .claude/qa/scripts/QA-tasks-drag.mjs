import { open } from './app.mjs';
const app = await open('tasks?project=none&view=board'); const p = app.page;
const col = (n)=>p.getByRole('region',{name:`Drop zone for ${n} tasks`});
const names = async()=>{const o={};for(const n of ['Backlog','In Progress','Done']){o[n]=(await col(n).locator('h4').allInnerTexts()).map(s=>s.slice(0,14))}return JSON.stringify(o)};
console.log('start', await names());
await app.shot('sweep-tasks-drag-fixtures-before');
const card=(t)=>p.getByRole('button',{name:`Select task: ${t}`});
await card('QA Beta').dragTo(col('In Progress')); await p.waitForTimeout(1000);
console.log('beta->IP', await names());
await card('QA Beta').dragTo(col('Done')); await p.waitForTimeout(1000);
console.log('beta->Done', await names());
await card('QA Beta').dragTo(col('Done')); await p.waitForTimeout(600); console.log('same col no-op', await names());
await card('QA Beta').dragTo(col('Backlog')); await p.waitForTimeout(1000);
console.log('beta->Backlog', await names());
await card('QA Alpha').dragTo(col('Done')); await p.waitForTimeout(1000);
console.log('alpha(parent w/ sub)->Done', await names());
await app.shot('sweep-tasks-drag-alpha-done');
// persistence
await p.reload(); await p.waitForTimeout(1500);
console.log('after reload', await names());
// detail
await p.goto(new URL('/tasks/67',p.url()).href); await p.waitForTimeout(1200);
console.log('ALPHA detail:', (await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,500));
await p.goto(new URL('/tasks/72',p.url()).href); await p.waitForTimeout(1200);
console.log('SUB detail:', (await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,300));
await app.done();
