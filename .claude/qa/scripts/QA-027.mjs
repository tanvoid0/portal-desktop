// Repro: a document card cannot be reached or opened by keyboard; its delete icon has no accessible name.
import { open } from './app.mjs';
const app = await open('documents');
const p = app.page;
await p.waitForTimeout(800);
const seen = [];
await p.locator('body').click({ position: { x: 700, y: 5 } });
for (let i = 0; i < 45; i++) {
  await p.keyboard.press('Tab');
  const d = await p.evaluate(() => { const e = document.activeElement; return `${e.tagName} ${(e.getAttribute('aria-label') || e.title || e.innerText || '').trim().slice(0, 30)} inMain=${!!e.closest('main')}`; });
  if (d.includes('inMain=true')) seen.push(d);
}
console.log('focusable in main (in order):\n' + seen.join('\n'));
console.log('card focusable/role:', await p.evaluate(() => { const c = document.querySelector('main [data-slot=card]'); return c ? `${c.getAttribute('role')} tabindex=${c.getAttribute('tabindex')}` : 'no card'; }));
console.log('delete icon name:', await p.evaluate(() => { const b = document.querySelector('main [data-slot=card] button'); return JSON.stringify(b?.getAttribute('aria-label') || b?.title || b?.innerText || ''); }));
await app.shot('QA-027-before-card-keyboard');
await app.done();
