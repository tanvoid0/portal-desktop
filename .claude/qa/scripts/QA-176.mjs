// Repro: the trash icon on /logs wipes every log line at once - no confirm, no toast, no undo.
import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('logs');
const p = app.page;
await dismissVault(p);
const count = () => p.evaluate(() => document.body.innerText.match(/Showing \d+ of \d+/)?.[0]);
console.log('before', await count());
await p.getByRole('button', { name: 'Clear logs' }).click();
await p.waitForTimeout(700);
console.log('dialogs', await p.locator('[role=alertdialog],[role=dialog]').count(), 'toasts', await p.locator('[data-sonner-toast]').count(), 'after', await count());
await app.done();
