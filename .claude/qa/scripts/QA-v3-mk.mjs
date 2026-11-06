import { open } from './app.mjs';
const specs = [
  { t: 'QA Nospace '+'A'.repeat(88), prio: 'High', type: null },
  { t: 'QA Cmp', prio: null, type: null },
  { t: 'QA Alpha', prio: 'High', type: 'Bug' },
  { t: 'QA Beta', prio: 'Low', type: 'Feature' },
];
for (const s of specs) {
  const app = await open('tasks/create'); const p = app.page;
  await p.fill('#title', s.t);
  if (s.prio) { await p.locator('button:has-text("Medium")').first().click(); await p.getByRole('option', { name: s.prio }).click(); }
  if (s.type) { await p.locator('button:has-text("Select type")').first().click(); await p.getByRole('option', { name: s.type }).click(); }
  await p.getByRole('button', { name: 'Create Task' }).click();
  await p.waitForTimeout(1500);
  console.log(s.t.slice(0,20), '->', p.url());
  await app.done();
}
