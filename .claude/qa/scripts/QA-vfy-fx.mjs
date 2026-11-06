import { open } from './app.mjs';
for (const t of ['QA Vfy Alpha','QA Vfy Bravo','QA Vfy Charlie','QA Vfy Delta']) {
  const app = await open('tasks/create'); const p = app.page;
  await p.fill('#title', t);
  await p.getByRole('button', { name: 'Create Task' }).click();
  await p.waitForTimeout(1500); console.log(t, p.url()); await app.done();
}
