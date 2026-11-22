// Repro: double-clicking Create in the "Create New Block" dialog makes two blocks. Cleans up after itself.
import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('automation/blocks');
const p = app.page;
await dismissVault(p);
await p.getByRole('button', { name: /^Create$/ }).first().click();
const dlg = p.getByRole('dialog');
await dlg.getByPlaceholder('e.g., Install Dependencies').fill('QA block dbl');
await dlg.getByPlaceholder('Describe what this block does.').fill('QA fixture');
await dlg.getByPlaceholder('e.g., npm install').fill('echo qa');
await dlg.getByRole('button', { name: /^Create$/ }).dblclick();
await p.waitForTimeout(1500);
await p.getByPlaceholder('Search blocks...').fill('QA block dbl');
await p.waitForTimeout(600);
const n = await p.getByRole('button', { name: /^Delete.*QA block dbl/ }).count().catch(() => 0);
const cards = await p.locator('[data-slot=card]', { hasText: 'QA block dbl' }).count();
console.log(cards > 1 ? `BUG: ${cards} blocks created from one double-click` : `ok: ${cards} block`);
for (let i = 0; i < cards; i++) {
  await p.locator('[data-slot=card]', { hasText: 'QA block dbl' }).first().getByRole('button').last().click();
  await p.waitForTimeout(500);
  await p.locator('[role=alertdialog],[role=dialog]').getByRole('button', { name: 'Delete' }).click();
  await p.waitForTimeout(900);
}
await app.done();
