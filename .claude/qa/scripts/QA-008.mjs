// QA-008: Settings > General "Theme" select does not reflect the theme chosen in the header.
import { open } from './app.mjs';
const app = await open('settings/general');
const p = app.page;
await p.getByRole('button', { name: 'Toggle theme' }).first().click();
await p.getByRole('menuitem', { name: 'Dark' }).click();
await p.waitForTimeout(700);
const cls = await p.evaluate(() => document.documentElement.className);
const sel = (await p.locator('button[role=combobox]').first().textContent()).trim();
console.log('header=Dark, html.class=' + cls + ', Settings Theme select shows:', sel, sel === 'Dark' ? 'ok' : 'FAIL');
await app.shot('QA-008-before-theme-select-out-of-sync');
await p.reload(); await p.waitForTimeout(1500);
console.log('after reload select shows:', (await p.locator('button[role=combobox]').first().textContent()).trim());
// restore
await p.getByRole('button', { name: 'Toggle theme' }).first().click();
await p.getByRole('menuitem', { name: 'System' }).click();
await p.waitForTimeout(700);
console.log('restored: html.class=' + (await p.evaluate(() => document.documentElement.className)));
await app.done();
