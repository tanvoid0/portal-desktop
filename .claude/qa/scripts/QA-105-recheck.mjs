// QA-105 re-verify: rapid-fire same-millisecond log entries should not crash the page.
import { open } from './app.mjs';

const app = await open('logs');
const page = app.page;

const themeToggle = page.getByRole('button', { name: 'Toggle theme' });
// Fire many toggles back-to-back with minimal delay to try to land two in the same ms.
for (let i = 0; i < 40; i++) {
  await themeToggle.click();
  await page.getByRole('menuitem', { name: i % 2 === 0 ? 'Light' : 'Dark' }).click();
}
await page.waitForTimeout(800);
const state = await page.evaluate(() => ({
  is500: document.body.innerText.includes('Something went wrong'),
  rows: document.querySelectorAll('div.font-mono').length,
  showing: (document.body.innerText.match(/Showing \d+ of \d+/) || [null])[0],
}));
console.log('after rapid toggles:', JSON.stringify(state));
await app.shot('QA-105-recheck-rapid-toggles');
await app.done();
