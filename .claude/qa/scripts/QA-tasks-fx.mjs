import { open } from './app.mjs';
const specs = [
  { t: 'QA Alpha', d: 'needle-alpha desc', prio: 'High', type: 'Bug' },
  { t: 'QA Beta', d: 'beta', prio: null, type: 'Feature' },
  { t: 'QA Gamma', d: '', prio: 'Low', type: null },
  { t: 'QA Nospace '+'A'.repeat(88), d: '', prio: 'High', type: null },
  { t: 'QA Cancelme', d: '', prio: null, type: null },
  { t: 'QA Sub 1', d: '', prio: null, type: null, parent: 'ALPHA' },
];
let alpha;
for (const s of specs) {
  const app = await open('tasks/create'+(s.parent?`?parentId=${alpha}`:''));
  const p = app.page;
  await p.fill('#title', s.t);
  if (s.d) await p.fill('#description', s.d);
  if (s.prio) { await p.locator('button:has-text("Medium")').first().click(); await p.getByRole('option', { name: s.prio }).click(); }
  if (s.type) { await p.locator('button:has-text("Select type")').first().click(); await p.getByRole('option', { name: s.type }).click(); }
  await p.getByRole('button', { name: 'Create Task' }).click();
  await p.waitForTimeout(1500);
  if (s.t==='QA Alpha') alpha = p.url().split('/').pop();
  console.log(s.t.slice(0,20), '->', p.url());
  await app.done();
}
