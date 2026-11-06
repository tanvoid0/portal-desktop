// Repro: saving an existing document shows the success toast twice.
import { open } from './app.mjs';
const app = await open('documents/1');
const p = app.page;
await p.waitForTimeout(800);
await p.getByRole('button', { name: 'Save Document' }).click();
await p.waitForTimeout(700);
console.log('"Document saved successfully" toasts:', await p.getByText('Document saved successfully').count(), '(expected 1)');
await app.shot('QA-029-before-double-toast');
await app.done();
