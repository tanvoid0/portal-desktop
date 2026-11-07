import { open } from './i2.mjs';
const app = await open('settings/general', { width: 1280, height: 800 });
const page = app.page;
await page.getByRole('button', { name: /toggle sidebar/i }).click();
await page.waitForTimeout(600);
await app.go('settings/general');   // full reload with the sidebar collapsed
const m = await page.evaluate(() => {
  const nav = [...document.querySelectorAll('nav')].pop();
  const btns=[...nav.querySelectorAll('button')];
  return { count: btns.length, w: Math.round(nav.getBoundingClientRect().width), first: btns[0]?.innerText.trim(), sidebarState: document.querySelector('[data-state]')?.getAttribute('data-state'),
           appSidebarW: Math.round((document.querySelector('[data-slot="sidebar-container"], [data-sidebar="sidebar"]')||document.body).getBoundingClientRect().width) };
});
console.log('after reload with app sidebar collapsed:', JSON.stringify(m));
await app.shot('UX-005-after-settings-nav-appsidebar-collapsed');
await page.getByRole('button', { name: /toggle sidebar/i }).click();
await page.waitForTimeout(500);
await app.done();
