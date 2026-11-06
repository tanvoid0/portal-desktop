// Repro: document draft auto-save always fails (arg name mismatch content_draft vs contentDraft).
import { open } from './app.mjs';
const app = await open('documents/1');
const p = app.page;
await p.waitForTimeout(800);
await p.getByPlaceholder(/document content/).fill('QA autosave probe ' + Date.now());
await p.waitForTimeout(3000); // autosave fires after 2s
const bad = app.errors.filter((e) => /Auto-save failed/.test(e));
console.log(bad.length ? 'BUG: ' + bad[0] : 'ok: no auto-save error');
await app.done();
