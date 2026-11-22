import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('database');
const p = app.page;
await dismissVault(p);
await p.waitForTimeout(1500);
await p.getByText('Portal (this app)').click();
await p.waitForTimeout(800);
await p.getByPlaceholder('Filter 53 tables...').fill('tasks');
await p.getByText(/^tasks$/).first().click();
await p.waitForTimeout(800);
await p.getByText('Query', { exact: true }).click();
const ta = p.getByPlaceholder('SELECT * FROM ...');
for (const [name, q] of [['bad','SELEC oops'],['noop-update','UPDATE tasks SET title=title WHERE id=-1'],['select','SELECT 1 AS a, \'日本語\' AS b']]) {
  await ta.fill(q);
  await p.getByRole('button', { name: 'Run' }).click();
  await p.waitForTimeout(1200);
  console.log(name, '=>', (await p.evaluate(()=>document.querySelector('main')?.innerText.split('Run').pop().replace(/\s+/g,' ').slice(0,200))));
}
console.log(await app.shot('sweep-database-query-result'));
await app.done();
