import { open } from './app.mjs';
for (const t of ['QA Del 1','QA Del 2','QA Del 3']) {
  const app = await open('tasks/create'); const p = app.page;
  await p.fill('#title', t); await p.getByRole('button',{name:'Create Task'}).click(); await p.waitForTimeout(1200);
  console.log(t, p.url()); await app.done();
}
