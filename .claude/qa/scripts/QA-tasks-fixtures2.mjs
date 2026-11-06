import { open } from './app.mjs';
const LONG = 'QA Long title '+'The quick brown fox jumps over the lazy dog and keeps running through the entire board layout '.repeat(3);
const NOSPACE = 'QA Nospace '+'A'.repeat(120);
const specs = [
  { t: 'QA Alpha', d: 'needle-alpha desc', prio: 'High', type: 'Bug' },
  { t: 'QA Beta', d: 'second one', prio: null, type: 'Feature' },
  { t: 'QA Gamma', d: '', prio: 'Low', type: null },
  { t: LONG, d: 'long one', prio: 'High', type: 'Bug' },
  { t: NOSPACE, d: '', prio: 'High', type: null },
  { t: 'QA Delta', d: '', prio: null, type: null },
  { t: 'QA Epsilon', d: '', prio: null, type: null },
];
for (const s of specs) {
  const app = await open('tasks/create');
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
