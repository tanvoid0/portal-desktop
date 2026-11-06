// Quick Run on an existing but unregistered folder -> FOREIGN KEY failure?
import { open } from './app.mjs';
import { mkdirSync } from 'node:fs';
const dir = String.raw`C:\Users\tanvo\AppData\Local\Temp\claude\0b7fd75641305543\3adf8854-41b4-4e9a-aca5-37f819e41735\scratchpad\qa-empty-dir`;
mkdirSync(dir, { recursive: true });
const app = await open('automation/run'); const p = app.page;
await p.waitForTimeout(1500);
await p.getByRole('button', { name: 'Test', exact: true }).click();
await p.getByPlaceholder('D:/projects/my-app').fill(dir);
await p.getByRole('button', { name: 'Run', exact: true }).click();
const s = new Set(); for (let i = 0; i < 15; i++) { await p.waitForTimeout(300); (await p.locator('[aria-live=polite] [role=alert]').allInnerTexts()).forEach(t => t.trim() && s.add(t.replace(/\s+/g, ' ').trim())); }
console.log('toasts:', [...s].join(' | ') || '(none)');
await app.shot('QA-071-before-quickrun-unregistered-folder');
await app.done();
