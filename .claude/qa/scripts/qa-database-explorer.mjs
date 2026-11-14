import { open } from './app.mjs';
const app = await open('database');
const p = app.page;
const log = (...a) => console.log(...a);
// check 1
log('source visible:', await p.getByText('Portal (this app)').count());
log('kind labels table:', await p.getByText('table', { exact: true }).count(), 'view:', await p.getByText('view', { exact: true }).count());
// check 2/3: try tables until one has next enabled
const names = ['agent_events','tasks','projects','agent_messages','ai_logs'];
let used;
for (const n of names) {
  await p.getByText(n, { exact: true }).first().click();
  await p.waitForTimeout(1200);
  const next = p.getByRole('button', { name: /next/i });
  log(n, 'next count', await next.count(), 'enabled', await next.count() ? await next.first().isEnabled() : null);
  if (await next.count() && await next.first().isEnabled()) { used = n; break; }
}
log('paged table:', used);
log('headers:', await p.locator('th').allInnerTexts());
log('PK markers:', await p.getByText('PK', { exact: true }).count());
const th = p.locator('th').first();
log('th title attr:', await th.getAttribute('title'), '| inner title:', await th.locator('[title]').first().getAttribute('title').catch(()=>null));
await th.hover(); await p.waitForTimeout(800);
log('tooltip:', await p.locator('[role=tooltip]').allInnerTexts());
const firstRow = () => p.locator('tbody tr').first().innerText();
log('rows:', await p.locator('tbody tr').count());
log('pager text:', await p.getByText(/\d+\s*[–-]\s*\d+|page/i).allInnerTexts());
if (used) {
  const r1 = await firstRow();
  await p.getByRole('button', { name: /next/i }).first().click(); await p.waitForTimeout(1000);
  const r2 = await firstRow(); log('next changed:', r1 !== r2);
  await p.getByRole('button', { name: /prev/i }).first().click(); await p.waitForTimeout(1000);
  log('prev restored:', (await firstRow()) === r1);
}
await app.shot('sweep-database-table-rows');
await app.done();
