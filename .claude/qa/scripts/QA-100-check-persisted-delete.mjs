import { open } from './app.mjs';
const app = await open('automation/blocks');
const page = app.page;
await page.waitForTimeout(1500);
await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);
console.log('present after fresh reload (post-delete):', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app.shot('QA-100-fresh-reload-post-delete');
await app.done();
