import { open } from './app.mjs';
const app = await open('database');
const p = app.page;
await p.waitForTimeout(1500);
await p.getByText('qa502.db', { exact: true }).first().click();
await p.waitForTimeout(400);
await p.getByText('qa_items', { exact: true }).click();
await p.waitForTimeout(1000);
await p.mouse.wheel(0, -5000);
await p.waitForTimeout(300);

const noteCell = () => p.locator('tbody tr').first().locator('td').nth(2);
const input = () => p.locator('input, textarea').first();

// edit + confirm
await noteCell().click();
await p.waitForTimeout(300);
await input().fill('QA edited value');
await p.keyboard.press('Enter');
await p.waitForTimeout(500);
await p.getByRole('button', { name: 'Confirm' }).click();
await p.waitForTimeout(800);
console.log('[after confirm] note cell text:', await noteCell().innerText());
await app.shot('QA-502-after-confirm-edit');

// reload page (navigate away and back) to prove persistence
const app2 = await open('database');
const p2 = app2.page;
await p2.waitForTimeout(1500);
await p2.getByText('qa502.db', { exact: true }).first().click();
await p2.waitForTimeout(400);
await p2.getByText('qa_items', { exact: true }).click();
await p2.waitForTimeout(1000);
console.log('[after reload] note cell text:', await p2.locator('tbody tr').first().locator('td').nth(2).innerText());
await app2.shot('QA-502-after-reload-persisted');

// now test Cancel: edit again, click Cancel, verify unchanged
await p2.mouse.wheel(0, -5000);
await p2.waitForTimeout(300);
const noteCell2 = () => p2.locator('tbody tr').first().locator('td').nth(2);
const input2 = () => p2.locator('input, textarea').first();
await noteCell2().click();
await p2.waitForTimeout(300);
await input2().fill('SHOULD BE CANCELLED');
await p2.keyboard.press('Enter');
await p2.waitForTimeout(500);
await p2.getByRole('button', { name: 'Cancel' }).click();
await p2.waitForTimeout(500);
console.log('[after cancel-dialog] note cell text:', await noteCell2().innerText());
await app2.shot('QA-502-after-cancel-dialog');
await app2.done();
