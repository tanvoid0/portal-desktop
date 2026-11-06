import { open } from './app.mjs';
const app = await open('tasks');
const p = app.page;
const cols = async () => p.evaluate(() => [...document.querySelectorAll('[aria-label^="Drop zone"]')].map(z => z.getAttribute('aria-label').replace('Drop zone for ','').replace(' tasks','') + ': ' + [...z.querySelectorAll('h4')].map(h=>h.textContent.trim()).filter(t=>t.startsWith('QA')).join(',')));
console.log('initial', await cols());
// search
const search = p.getByPlaceholder('Search tasks...');
await search.fill('needle-alpha');
await p.waitForTimeout(300);
console.log('search needle (desc):', await cols());
await search.fill('qa beta');
console.log('search case-insens:', await cols());
await search.fill('zzzznomatch');
await p.waitForTimeout(300);
console.log('no match text:', (await p.locator('main').innerText()).includes('No matching tasks'));
await app.shot('sweep-tasks-no-match');
await p.getByRole('button', { name: 'Clear filters' }).click();
console.log('after clear:', await cols(), 'search value=', await search.inputValue());
await search.fill('QA');
await search.press('Escape');
console.log('esc in search value=', await search.inputValue());
await search.fill('');
// drag Alpha -> In Progress
const card = p.locator('[draggable=true]', { hasText: 'QA Alpha' }).first();
const zone = p.locator('[aria-label="Drop zone for In Progress tasks"]');
await card.dragTo(zone);
await p.waitForTimeout(1500);
console.log('after drag->in progress:', await cols());
console.log('loading flash? n/a');
// reload persists
await p.reload(); await p.waitForTimeout(1200);
console.log('after reload:', await cols());
await app.shot('sweep-tasks-after-drag');
// drag back to todo
await p.locator('[draggable=true]', { hasText: 'QA Alpha' }).first().dragTo(p.locator('[aria-label="Drop zone for To Do tasks"]'));
await p.waitForTimeout(1500);
console.log('after drag back:', await cols());
// drag onto same column (no-op)
await p.locator('[draggable=true]', { hasText: 'QA Alpha' }).first().dragTo(p.locator('[aria-label="Drop zone for To Do tasks"]'));
await p.waitForTimeout(800);
console.log('same col:', await cols());
await app.done();
