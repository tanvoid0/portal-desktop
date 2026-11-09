import { open } from './app.mjs';
const app = await open('automation/blocks');
const page = app.page;
await page.waitForTimeout(1200);
await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);
console.log('QA Imported Template present:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app.shot('qa-final-cleanup-check-templates');
await app.done();
