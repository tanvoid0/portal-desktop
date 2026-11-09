import { open } from './app.mjs';

const app = await open('automation/blocks');
const page = app.page;
await page.setViewportSize({ width: 1024, height: 600 });
await page.waitForTimeout(1000);

const titles = await page.evaluate(() => {
  const cards = [...document.querySelectorAll('[data-slot="card"]')];
  return cards.slice(0, 12).map((c) => {
    const titleEl = c.querySelector('h3, [class*="font-"]');
    return titleEl ? titleEl.textContent.trim() : '(no title found)';
  });
});
console.log('titles at 1024x600:', JSON.stringify(titles));
await app.shot('QA-101-blocks-1024x600');

await page.setViewportSize({ width: 1280, height: 800 });
await page.waitForTimeout(500);
await app.shot('QA-101-blocks-1280');
await app.done();
