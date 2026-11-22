// Repro: a long unbroken script name overflows the card + detail header and makes the pane scroll sideways.
// Creates "QA script long" and deletes it again.
import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('scripts/new');
const p = app.page;
await dismissVault(p);
await p.getByPlaceholder('e.g., VPN Connect').fill('QA script long ' + 'x'.repeat(90));
await p.getByPlaceholder('Describe what this script does').fill('QA fixture');
await p.getByPlaceholder(/e.g., \.\/scripts/).fill('echo qa');
await p.getByRole('button', { name: 'Save Script' }).click();
await p.waitForTimeout(1500);
await p.goto(new URL('/automation/scripts', p.url()).href); await dismissVault(p, 1500);
await p.getByPlaceholder('Search scripts...').fill('QA script long');
await p.waitForTimeout(600);
const res = await p.evaluate(() => {
  const wide = [...document.querySelectorAll('*')].filter(e => e.scrollWidth > e.clientWidth + 1 && ['auto', 'scroll'].includes(getComputedStyle(e).overflowX));
  return { overflows: wide.length > 0, scrollers: wide.map(e => e.tagName + ' ' + e.scrollWidth + '>' + e.clientWidth).slice(0, 3) };
});
console.log(res.overflows ? 'BUG: pane scrolls sideways ' + JSON.stringify(res) : 'ok ' + JSON.stringify(res));
console.log(await app.shot('QA-169-run-long-name'));
for (let i = 0; i < 5; i++) { const d = p.getByRole('button', { name: /^Delete script QA script long/ }); if (!(await d.count())) break; await d.first().click();
await p.locator('[role=alertdialog],[role=dialog]').getByRole('button', { name: 'Delete' }).click(); await p.waitForTimeout(800); }
await app.done();
