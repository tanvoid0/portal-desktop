import { open } from './app.mjs';
let app = await open('tasks/generate'); let p = app.page;
console.log((await p.locator('main').innerText()).slice(0,400));
await app.shot('sweep-tasks-generate');
for (const id of [52,53,54]) {
  await p.goto(new URL('/tasks/'+id, p.url()).href); await p.waitForTimeout(1000);
  await p.getByRole('button',{name:'Delete'}).first().click(); await p.waitForTimeout(400);
  await p.getByRole('alertdialog').getByRole('button',{name:'Delete'}).click().catch(async()=>p.getByRole('dialog').getByRole('button',{name:'Delete'}).click());
  await p.waitForTimeout(1500); console.log('deleted', id, '->', p.url());
}
await p.goto(new URL('/tasks/55', p.url()).href); await p.waitForTimeout(1000);
console.log('sub 55 after parent delete:', (await p.locator('main').innerText()).slice(0,150).replace(/\n/g,' | '));
await p.goto(new URL('/tasks', p.url()).href); await p.waitForTimeout(1000);
console.log('QA left:', await p.locator('h4').allInnerTexts().then(a=>a.filter(t=>t.startsWith('QA '))));
await app.done();
