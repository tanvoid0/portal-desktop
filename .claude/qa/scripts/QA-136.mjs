// Repro: saving a template pipeline creates TWO rows. Creates fixtures named "QA dup", deletes them.
import { open } from './app.mjs';
const app = await open('projects/14/pipelines/new'); const p = app.page;
await p.waitForTimeout(3000);
await p.locator('button', { hasText: 'All Templates' }).click(); await p.waitForTimeout(1000);
await p.getByText('Docker Build & Deploy', { exact: true }).first().click(); await p.waitForTimeout(2000);
await p.getByLabel('Pipeline Name').fill('QA dup');
const count = () => p.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('get_pipelines', { projectId: 14 })).filter(x => x.name === 'QA dup').length);
await p.locator('button', { hasText: 'Save Pipeline' }).click(); await p.waitForTimeout(3000);
const n = await count();
console.log(n === 1 ? 'OK: 1 row' : `BUG: ${n} rows named "QA dup" after a single Save`);
await p.evaluate(async () => { for (const x of (await window.__TAURI_INTERNALS__.invoke('get_pipelines', { projectId: 14 })).filter(x => x.name === 'QA dup')) await window.__TAURI_INTERNALS__.invoke('delete_pipeline', { pipelineId: x.id }); });
await app.done();
