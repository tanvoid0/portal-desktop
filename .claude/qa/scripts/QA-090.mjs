// QA-090 verify: log viewer should return to live tail on demand, pause auto-scroll
// when the user scrolls away, and offer a "Scroll to bottom" control.
// Generates log volume in-place (toggling the theme logs "Theme changed to:" each time)
// so we never navigate away from /logs and never touch browser history depth.
import { open } from './app.mjs';

const app = await open('logs', { width: 1280, height: 800 });
const page = app.page;

const themeToggle = page.getByRole('button', { name: 'Toggle theme' });
for (let i = 0; i < 150; i++) {
  await themeToggle.click();
  await page.getByRole('menuitem', { name: i % 2 === 0 ? 'Light' : 'Dark' }).click();
  await page.waitForTimeout(60);
}
const showingText = await page.getByText(/Showing \d+ of \d+/).textContent().catch(() => 'N/A');
console.log('after 60 toggles, showing:', showingText);

async function findScrollableViewport() {
  const infos = await page
    .locator('[data-slot="scroll-area-viewport"]')
    .evaluateAll((els) => els.map((e) => ({ scrollHeight: e.scrollHeight, clientHeight: e.clientHeight })));
  const idx = infos.findIndex((v) => v.scrollHeight > v.clientHeight + 10);
  return { idx, infos };
}
const { idx, infos } = await findScrollableViewport();
console.log('viewport infos:', JSON.stringify(infos), 'idx:', idx);
if (idx === -1) {
  console.log('NEVER OVERFLOWED - cannot test scroll behavior');
  await app.done();
  process.exit(1);
}
const viewport = page.locator('[data-slot="scroll-area-viewport"]').nth(idx);

const stats = async () =>
  viewport.evaluate((sc) => ({
    total: document.querySelectorAll('div.font-mono').length,
    scrollTop: Math.round(sc.scrollTop),
    scrollMax: Math.round(sc.scrollHeight - sc.clientHeight),
  }));

console.log('before scroll-up:', JSON.stringify(await stats()));

await viewport.hover();
for (let i = 0; i < 10; i++) {
  await page.mouse.wheel(0, -2000);
  await page.waitForTimeout(80);
}
await page.waitForTimeout(500);
console.log('after scrolling to top:', JSON.stringify(await stats()));

const autoScrollCb = page.locator('#log-auto-scroll');
const autoState = await page.evaluate(() => document.querySelector('#log-auto-scroll')?.getAttribute('data-state'));
console.log('auto-scroll data-state after manual scroll-up (expect unchecked):', autoState);

const scrollBtn = page.getByRole('button', { name: /scroll to bottom/i });
console.log('"Scroll to bottom" button present after scrolling up:', await scrollBtn.count());
await app.shot('QA-090-after-scroll-up-button-visible');

if (await scrollBtn.count()) {
  await scrollBtn.first().click();
  await page.waitForTimeout(500);
  console.log('after clicking Scroll to bottom:', JSON.stringify(await stats()));
  console.log('"Scroll to bottom" button present after click (expect gone):', await scrollBtn.count());
}

// Scroll up again, then re-check Auto-scroll box, expect immediate jump to bottom.
await viewport.hover();
for (let i = 0; i < 10; i++) {
  await page.mouse.wheel(0, -2000);
  await page.waitForTimeout(80);
}
await page.waitForTimeout(500);
console.log('scrolled up again:', JSON.stringify(await stats()));
await autoScrollCb.click();
await page.waitForTimeout(500);
const stateAfterClick = await page.evaluate(() => document.querySelector('#log-auto-scroll')?.getAttribute('data-state'));
console.log('auto-scroll data-state after manual re-check:', stateAfterClick);
console.log('after re-checking Auto-scroll (expect scrollTop==scrollMax):', JSON.stringify(await stats()));
await app.shot('QA-090-after-autoscroll-recheck');

await app.done();
