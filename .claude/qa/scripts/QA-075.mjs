// Sidebar footer icon buttons: names + tab stops before page content
import { open } from './app.mjs';
const app = await open('projects'); const p = app.page;
await p.waitForTimeout(1500);
const foot = await p.evaluate(() => [...document.querySelectorAll('[data-sidebar=footer] button, [data-sidebar=footer] a, aside button, aside a')].filter(x => x.getBoundingClientRect().top > 720).map(x => x.tagName + ' name=' + JSON.stringify(x.getAttribute('aria-label') || x.title || x.innerText.trim())));
console.log('footer controls near bottom:', foot);
await p.locator('body').click({ position: { x: 640, y: 5 } });
let n = 0;
for (; n < 80; n++) { await p.keyboard.press('Tab'); const inMain = await p.evaluate(() => !!document.activeElement.closest('main')); if (inMain) break; }
console.log('Tab presses to reach first control inside <main>:', n + 1);
await app.done();
