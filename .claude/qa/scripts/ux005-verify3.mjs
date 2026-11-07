import { open } from './i2.mjs';
const app = await open('settings/general', { width: 1280, height: 800 });
const page = app.page;

// keyboard: focus the first nav item and Enter
await page.evaluate(() => {
  const nav = [...document.querySelectorAll('nav')].pop();
  nav.querySelectorAll('button')[1].focus();   // Updates
});
const focused = await page.evaluate(() => ({ tag: document.activeElement.tagName, text: document.activeElement.innerText.trim(), outline: getComputedStyle(document.activeElement).outlineWidth }));
console.log('focused:', JSON.stringify(focused));
await page.keyboard.press('Enter');
await page.waitForTimeout(900);
console.log('after Enter url:', page.url(), 'h1:', await page.evaluate(()=>document.querySelector('h1')?.innerText.trim()));
// Tab to next and Space
await page.keyboard.press('Tab');
const f2 = await page.evaluate(() => document.activeElement.innerText.trim());
console.log('tab ->', JSON.stringify(f2));
await page.keyboard.press('Space');
await page.waitForTimeout(900);
console.log('after Space url:', page.url());

// collapsed rail
await app.go('settings/general');
await page.getByRole('button', { name: /toggle sidebar/i }).click();
await page.waitForTimeout(900);
await app.shot('UX-005-after-settings-rail-collapsed');
const rail = await page.evaluate(() => {
  const nav = [...document.querySelectorAll('nav')].pop();
  const btns = [...nav.querySelectorAll('button')];
  return { count: btns.length, w: Math.round(nav.getBoundingClientRect().width), texts: btns.map(b=>b.innerText.trim()) };
});
console.log('collapsed rail nav:', JSON.stringify(rail));
// hover first rail item for a tooltip
const first = page.locator('nav button').last();
await page.evaluate(() => { const nav=[...document.querySelectorAll('nav')].pop(); nav.querySelectorAll('button')[0].dispatchEvent(new PointerEvent('pointerenter',{bubbles:true})); });
await page.locator('nav >> button').nth(0).hover().catch(()=>{});
await page.waitForTimeout(900);
const tip = await page.evaluate(() => [...document.querySelectorAll('[role="tooltip"],[data-slot="tooltip-content"]')].map(e=>e.innerText.trim()));
console.log('tooltip:', JSON.stringify(tip));
await app.shot('UX-005-after-settings-rail-tooltip');
await page.getByRole('button', { name: /toggle sidebar/i }).click();
await page.waitForTimeout(600);
await app.done();
