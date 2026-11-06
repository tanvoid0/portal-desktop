import { open } from './app.mjs';
const specs = [
  { t: ('QA Long '+'quick brown fox jumps over the lazy dog and keeps running through the board ').slice(0,99), d: 'long one', prio: 'High', type: 'Bug', tags: 'verylongtagname-aaaaaaaaaaaaaaaaaaaa, t2' },
  { t: 'QA Nospace '+'A'.repeat(88), d: '', prio: 'High', type: null },
  { t: 'QA Sub 1', d: '', prio: null, type: null, parent: 58 },
];
for (const s of specs) {
  const app = await open('tasks/create'+(s.parent?`?parentId=${s.parent}`:''));
  const p = app.page;
  await p.fill('#title', s.t);
  if (s.d) await p.fill('#description', s.d);
  if (s.prio) { await p.locator('button:has-text("Medium")').first().click(); await p.getByRole('option', { name: s.prio }).click(); }
  if (s.type) { await p.locator('button:has-text("Select type")').first().click(); await p.getByRole('option', { name: s.type }).click(); }
  await p.getByRole('button', { name: 'Create Task' }).click();
  await p.waitForTimeout(1500);
  console.log(s.t.slice(0,20), '->', p.url());
  await app.done();
}
