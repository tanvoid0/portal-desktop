// QA-011: sidebar footer theme toggle is a button nested in a button: two tab stops, one control.
import { open } from './app.mjs';
const app = await open('');
const r = await app.page.getByRole('button', { name: 'Toggle theme' }).evaluateAll(els => els.map(e => e.tagName + ' nestedButtons=' + e.querySelectorAll('button').length + ' parentIsButton=' + !!e.parentElement.closest('button')));
console.log(r);
await app.page.locator('body').focus().catch(()=>{});
const stops = [];
await app.page.evaluate(() => document.activeElement?.blur());
for (let i = 0; i < 30; i++) { await app.page.keyboard.press('Tab'); stops.push(await app.page.evaluate(() => (document.activeElement.textContent || '').trim())); }
console.log('Tab stops labelled "Toggle theme":', stops.filter(s => s === 'Toggle theme').length, '(expected 1)');
await app.shot('QA-011-before-footer-theme-toggle');
await app.done();
