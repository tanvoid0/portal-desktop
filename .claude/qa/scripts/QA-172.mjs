// Repro: Escape does not close the open task rail; Back is the only way out.
import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('tasks');
const p = app.page;
await dismissVault(p);
await p.getByText(/release_rehearse refuses/).first().click();
await p.waitForTimeout(800);
console.log('open:', p.url());
await p.keyboard.press('Escape'); await p.waitForTimeout(600);
console.log(/task=/.test(p.url()) ? 'BUG: rail still open after Esc ' + p.url() : 'ok: closed');
await app.done();
