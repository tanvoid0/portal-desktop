import { open } from './app.mjs';
const app = await open('database');
const p = app.page;
await p.waitForTimeout(1500);

// Find qa502.db's editing toggle and ensure it is OFF.
const qa502Text = p.getByText('qa502.db', { exact: true }).first();
await qa502Text.scrollIntoViewIfNeeded();
await p.waitForTimeout(300);
const toggle = qa502Text.locator('xpath=ancestor::*[self::li or self::div][1]//*[@role="switch"]').first();
const checked = await toggle.getAttribute('aria-checked');
console.log('qa502.db editing toggle currently:', checked);
if (checked === 'true') {
  await toggle.click();
  await p.waitForTimeout(400);
  console.log('toggled off, now:', await toggle.getAttribute('aria-checked'));
}

// Get the qa502.db source id by calling db_list_sources directly.
const result = await p.evaluate(async () => {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  const sources = await invoke('db_list_sources');
  const src = sources.find((s) => s.label === 'qa502.db');
  if (!src) return { error: 'qa502.db source not found', sources };
  try {
    const r = await invoke('db_apply_update', {
      sourceId: src.id,
      table: 'qa_items',
      pkColumn: 'id',
      pkValue: 1,
      column: 'note',
      newValue: 'HACKED VIA DIRECT INVOKE',
    });
    return { unexpectedSuccess: r, editableFlag: src.editable };
  } catch (e) {
    return { refusedWith: String(e), editableFlag: src.editable };
  }
});
console.log(JSON.stringify(result, null, 2));
await app.done();
