import { open } from './app.mjs';
const app = await open('tasks'); const p = app.page;
for (const id of [58,59,60,61,62,63,64]) {
  await p.goto(new URL('/tasks/'+id, p.url()).href); await p.waitForTimeout(900);
  const t = (await p.locator('main').innerText().catch(()=>'')).replace(/^[sS]*?(QA )/,'$1');
  
  await p.getByRole('button',{name:'Delete'}).first().click(); await p.waitForTimeout(400);
  await p.getByRole('alertdialog').getByRole('button',{name:'Delete'}).click().catch(async()=>p.getByRole('dialog').getByRole('button',{name:'Delete'}).click());
  await p.waitForTimeout(1200); console.log('deleted',id,t.slice(0,20));
}
await p.goto(new URL('/tasks?project=none', p.url()).href); await p.waitForTimeout(1200);
console.log('QA left:', (await p.locator('main').innerText()).match(/QA [^\n]*/g));
await app.done();
