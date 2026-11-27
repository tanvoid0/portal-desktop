// series colors, history after client-side nav, sidebar link (expanded/collapsed, click/keyboard), disk dashboard
import { open } from './app.mjs';
const app = await open('system', { });
const { page } = app;
await page.setViewportSize({ width: 1280, height: 800 });
const wasDark = await page.evaluate(() => document.documentElement.classList.contains('dark'));
await page.waitForTimeout(65000);
const colors = () => page.evaluate(() => [...document.querySelectorAll('polyline')].map((p) => getComputedStyle(p).stroke));
const lines = () => page.evaluate(() => [...document.querySelectorAll('polyline')].map((p) => { const a = p.getAttribute('points').split(' '); return `${a.length} pts x=${(+a[0].split(',')[0]).toFixed(1)}..${(+a.at(-1).split(',')[0]).toFixed(1)}`; }));
for (const d of [false, true]) { await page.evaluate((d) => document.documentElement.classList.toggle('dark', d), d); console.log(d ? 'dark' : 'light', await colors()); }
console.log('after 65s', await lines());
console.log(await app.shot('sweep-system-chart-full-window-dark'));
await page.evaluate((d) => document.documentElement.classList.toggle('dark', d), wasDark);

// away (client-side) for 70s, back through the sidebar link
const link = page.locator('a[href="/system"]').filter({ hasText: 'Open System page' });
await page.locator('a[href="/settings"]').first().click();
await page.waitForTimeout(70000);
console.log('on', page.url(), 'sidebar link count', await link.count(), JSON.stringify(await link.ariaSnapshot()));
await link.click();
await page.waitForTimeout(150);
console.log('click ->', page.url(), 'arrival', await lines());
console.log(await app.shot('sweep-system-return-after-70s'));
await page.waitForTimeout(3000);
console.log('3s later', await lines());

// keyboard: Tab to the link from the top, Enter
async function viaKeyboard(label) {
  await page.locator('a[href="/settings"]').first().click();
  await page.waitForTimeout(800);
  await page.evaluate(() => document.activeElement.blur());
  let n = 0, hit = false;
  while (n++ < 60 && !hit) { await page.keyboard.press('Tab'); hit = await page.evaluate(() => /Open System page/.test(document.activeElement?.textContent ?? '') && document.activeElement.tagName === 'A'); }
  console.log(label, 'reached by Tab:', hit, n);
  console.log(await app.shot(`sweep-system-sidebar-focus-${label}`));
  await page.keyboard.press('Enter');
  await page.waitForTimeout(800);
  console.log(label, 'Enter ->', page.url(), JSON.stringify(await link.boundingBox()));
}
await viaKeyboard('expanded');
await page.keyboard.press('Control+b');
await page.waitForTimeout(700);
console.log('collapsed state', await page.evaluate(() => document.querySelector('[data-collapsible=icon]') ? 'icon' : 'not collapsed'));
await viaKeyboard('collapsed');
await page.locator('a[href="/settings"]').first().click();
await page.waitForTimeout(800);
await link.click();
await page.waitForTimeout(800);
console.log('collapsed click ->', page.url());
console.log(await app.shot('sweep-system-sidebar-collapsed'));
await page.keyboard.press('Control+b');
await page.waitForTimeout(500);

await page.goto(new URL('/utilities/disk', page.url()).href);
await page.waitForTimeout(6000);
console.log(await app.shot('sweep-utilities-disk-drives'));
console.log((await page.locator('body').innerText()).split('\n').slice(-40).join(' / '));
await app.done();
