// QA-064: deleting a credential in the UI only removes it from the local store; it is back after reload.
// Creates 'QA credential' (fake value), deletes it via UI, reloads, then removes the leftover via invoke (fixture cleanup).
import { open } from './app.mjs';
const app = await open('credentials'); const p = app.page; await p.waitForTimeout(1500);
await p.getByRole('button', { name: 'Add Credential' }).click();
const d = p.locator('[role=dialog]');
await d.locator('#name').fill('QA credential'); await d.locator('#value').fill('qa-fake-value');
await d.getByRole('button', { name: 'Create' }).click(); await p.waitForTimeout(1500);
await p.getByRole('button', { name: 'Delete QA credential' }).click(); await p.waitForTimeout(600);
await p.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click(); await p.waitForTimeout(1200);
console.log('gone right after delete:', (await p.locator('main button[aria-label="Delete QA credential"]').count()) === 0);
await p.reload(); await p.waitForTimeout(2000);
console.log('still present after reload (BUG if 1):', await p.locator('main button[aria-label="Delete QA credential"]').count());
await app.shot('QA-064-before-credential-back-after-reload');
await p.evaluate(async () => { const inv = window.__TAURI_INTERNALS__.invoke; for (const c of (await inv('get_credentials')).filter(c => c.name === 'QA credential')) await inv('delete_credential', { id: c.id }); });
await app.done();
