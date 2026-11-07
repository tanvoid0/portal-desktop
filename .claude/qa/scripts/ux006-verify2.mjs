// UX-006 part 2: real line capacity with an overflowing buffer, auto-scroll behaviour,
// and whether re-checking Auto-scroll replaces the deleted "Scroll to bottom" button.
import { open } from './i2.mjs';

const app = await open('logs', { width: 1280, height: 800 });
const page = app.page;

// Client-side navigation keeps the frontend logger singleton alive, so entries accumulate.
const links = ['Overview', 'Tasks', 'Documents', 'Credentials', 'Action Runs', 'Projects'];
let clicks = 0;
for (let pass = 0; pass < 5; pass++) {
  for (const name of links) {
    const l = page.getByRole('link', { name, exact: true }).first();
    if (await l.count()) {
      await l.click().catch(() => {});
      await page.waitForTimeout(180);
      clicks++;
    }
  }
}
// history.go back to /logs — still a client-side navigation, so the logger keeps its entries.
await page.evaluate((n) => history.go(-n), clicks);
await page.waitForTimeout(2000);
console.log('url now:', page.url(), 'clicks:', clicks);

const stats = async () =>
  page.evaluate(() => {
    const rows = [...document.querySelectorAll('div.font-mono')];
    const vh = window.innerHeight;
    const vis = rows.filter((r) => {
      const b = r.getBoundingClientRect();
      return b.top >= 0 && b.bottom <= vh + 1 && b.height > 0;
    });
    let sc = rows[0];
    while (sc && sc.scrollHeight <= sc.clientHeight + 2) sc = sc.parentElement;
    return {
      total: rows.length,
      visible: vis.length,
      rowH: rows[0] ? Math.round(rows[0].getBoundingClientRect().height) : null,
      firstRowTop: rows[0] ? Math.round(rows[0].getBoundingClientRect().top) : null,
      scrollTop: sc ? Math.round(sc.scrollTop) : null,
      scrollMax: sc ? Math.round(sc.scrollHeight - sc.clientHeight) : null,
      viewportBottomGap: sc ? Math.round(vh - sc.getBoundingClientRect().bottom) : null,
      showing: (document.body.innerText.match(/Showing \d+ of \d+/) || [null])[0],
      vh,
    };
  });

console.log('1280x800 on mount:', JSON.stringify(await stats()));
await app.shot('UX-006-after-logs-full-1280');

// scroll up, then toggle Auto-scroll off and back on: does it return to the bottom?
await page.evaluate(() => {
  const rows = [...document.querySelectorAll('div.font-mono')];
  let sc = rows[0];
  while (sc && sc.scrollHeight <= sc.clientHeight + 2) sc = sc.parentElement;
  if (sc) sc.scrollTop = 0;
});
await page.waitForTimeout(300);
console.log('after scrolling to top:', JSON.stringify(await stats()));
const cb = page.locator('#log-auto-scroll');
await cb.click();
await page.waitForTimeout(200);
await cb.click(); // re-enable
await page.waitForTimeout(600);
console.log('after re-checking Auto-scroll:', JSON.stringify(await stats()));
console.log('"Scroll to bottom" control present?', await page.getByRole('button', { name: /scroll to bottom/i }).count());
await app.shot('UX-006-after-logs-autoscroll-recheck');

// 1024x600
await app.size(1024, 600);
await page.waitForTimeout(500);
console.log('1024x600:', JSON.stringify(await stats()));
await app.shot('UX-006-after-logs-1024x600');
await app.size(1280, 800);
await app.done();
