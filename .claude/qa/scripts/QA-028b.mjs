// Repro: the "Archived" checkbox is not persisted (is_archived sent snake_case; Tauri expects isArchived).
import { open } from './app.mjs';
const app = await open('documents/1');
const p = app.page;
await p.waitForTimeout(800);
const cb = p.getByRole('checkbox');
await cb.click();
await p.getByRole('button', { name: 'Save Document' }).click();
await p.waitForTimeout(1200);
const rows = await p.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('get_documents')).map((d) => [d.id, d.title, d.is_archived]));
console.log('stored is_archived for fixture:', JSON.stringify(rows), '(expected true)');
await app.shot('QA-028-before-archived-not-saved');
await app.done();
