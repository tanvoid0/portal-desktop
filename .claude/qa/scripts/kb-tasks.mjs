import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('tasks');
const p = app.page;
await dismissVault(p);
// first card wrapper
const info = await p.evaluate(() => { const c = document.querySelector('[data-testid^=task],[role=button][tabindex="0"]'); c?.focus(); return c ? c.outerHTML.slice(0,160) : null; });
console.log('card', info);
await p.keyboard.press('Enter'); await p.waitForTimeout(800);
console.log('after Enter', p.url());
await p.keyboard.press('Escape'); await p.waitForTimeout(500);
console.log('after Esc', p.url(), 'rail present', await p.getByText('Task Details').count());
await p.keyboard.press('Shift+Slash').catch(()=>{});
await p.keyboard.press('?'); await p.waitForTimeout(500);
console.log('? opens shortcuts', await p.locator('[role=dialog]').count());
console.log(await app.shot('sweep-tasks-keyboard-shortcuts'));
await app.done();
