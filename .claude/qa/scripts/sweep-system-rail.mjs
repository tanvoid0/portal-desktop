// collapsed rail at the minimum window
import { open } from './app.mjs';
const app = await open('system');
const { page } = app;
await page.setViewportSize({ width: 1024, height: 600 });
await page.keyboard.press('Control+b');
await page.waitForTimeout(2500);
console.log(await app.shot('sweep-system-sidebar-collapsed-min'));
console.log(await page.evaluate(() => [...document.querySelectorAll('[data-slot=sidebar-content], [data-sidebar=content], [data-sidebar=content] *')].filter((e) => e.scrollWidth > e.clientWidth + 1 && /auto|scroll/.test(getComputedStyle(e).overflowX)).map((e) => `${e.tagName} ${e.scrollWidth}>${e.clientWidth}`)));
await page.keyboard.press('Control+b');
await page.waitForTimeout(500);
await page.setViewportSize({ width: 1280, height: 800 });
await app.done();
