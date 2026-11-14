import { open } from './app.mjs';
const app = await open('database');
const p = app.page;
await p.waitForTimeout(1500);

// First-load empty state
await app.shot('UX-sweep-database-initial-state');

await p.getByText('qa502.db', { exact: true }).first().click();
await p.waitForTimeout(400);
await p.getByText('qa_items', { exact: true }).click();
await p.waitForTimeout(800);
await p.mouse.wheel(0, -5000);
await p.waitForTimeout(300);

// Switch to Query tab, type SQL
await p.getByRole('tab', { name: 'Query' }).click().catch(() => p.getByText('Query', { exact: true }).click());
await p.waitForTimeout(400);
const editor = p.locator('textarea').first();
await editor.fill('SELECT * FROM qa_items WHERE 1=1 -- my scratch query');
await app.shot('UX-sweep-query-tab-typed');

// Switch table (Data tab, qa_no_pk), then back to Query tab: is SQL text preserved?
await p.getByRole('tab', { name: 'Data' }).click().catch(() => p.getByText('Data', { exact: true }).click());
await p.waitForTimeout(300);
await p.getByText('qa_no_pk', { exact: true }).click();
await p.waitForTimeout(500);
await p.mouse.wheel(0, -5000);
await p.waitForTimeout(300);
await p.getByRole('tab', { name: 'Query' }).click().catch(() => p.getByText('Query', { exact: true }).click());
await p.waitForTimeout(400);
const textAfterSwitch = await p.locator('textarea').first().inputValue();
console.log('query text survived table switch:', textAfterSwitch);
await app.shot('UX-sweep-query-tab-after-table-switch');

await app.done();
