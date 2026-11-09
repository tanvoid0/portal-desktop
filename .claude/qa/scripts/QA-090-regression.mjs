// QA-090 reopen evidence: the fix's `<ScrollArea class="absolute inset-0">` cannot actually
// become absolutely positioned, because bits-ui's ScrollArea Root sets an inline
// `style="position: relative"` that always beats any position utility class. The pane no
// longer fills available height or scrolls — it grows with content until clipped by the
// page shell's `overflow-hidden`, so once there are enough entries the newest ones are
// simply invisible, with no scrollbar and no "Scroll to bottom" button (isAtBottom never
// goes false because nothing ever scrolls).
import { open } from './app.mjs';

const app = await open('logs', { width: 1280, height: 800 });
const page = app.page;

const themeToggle = page.getByRole('button', { name: 'Toggle theme' });
for (let i = 0; i < 80; i++) {
  await themeToggle.click();
  await page.getByRole('menuitem', { name: i % 2 === 0 ? 'Light' : 'Dark' }).click();
  await page.waitForTimeout(50);
}

const info = await page.evaluate(() => {
  const root = document.querySelector('[data-slot="scroll-area"]');
  const vp = document.querySelector('[data-slot="scroll-area-viewport"]');
  const main = document.querySelector('main');
  return {
    inlineStyle: root?.getAttribute('style'),
    computedPosition: root ? getComputedStyle(root).position : null,
    vpScrollHeight: vp?.scrollHeight,
    vpClientHeight: vp?.clientHeight,
    mainOverflow: main ? getComputedStyle(main).overflow : null,
    totalRows: document.querySelectorAll('div.font-mono').length,
    showing: (document.body.innerText.match(/Showing \d+ of \d+/) || [null])[0],
  };
});
console.log(JSON.stringify(info, null, 2));
console.log(
  '"Scroll to bottom" button present (should exist, content overflows):',
  await page.getByRole('button', { name: /scroll to bottom/i }).count()
);
await app.shot('QA-090-regression-entries-clipped-not-scrollable');
await app.done();
