// QA-088: block + template import actually open a picker and create the item; bad files error.
import { open } from './i2.mjs';
const DIR = 'C:/Users/tanvo/AppData/Local/Temp/claude/660cd8302a863ca1/2609a446-530c-4f61-83da-a545cae02459/scratchpad/qafiles/';
const app = await open('automation/blocks', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(2500);
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const count = () => page.evaluate(() => [...document.querySelectorAll('[data-slot="card"]')].filter(c=>c.getBoundingClientRect().y>150).length);

console.log('blocks before:', await count());
await page.getByRole('button', { name: 'Import', exact: true }).click();
await page.waitForTimeout(600);
const inputs = await page.locator('[role="dialog"] input[type=file]').count();
console.log('file input present in the Import dialog:', inputs);
await page.locator('[role="dialog"] input[type=file]').setInputFiles(DIR + 'block-good.json');
await page.waitForTimeout(2000);
console.log('toast:', await toastText());
console.log('blocks after good import:', await count());
console.log('QA Imported Block present:', (await page.innerText('body')).includes('QA Imported Block'));
await app.shot('QA-088-after-block-import');

// bad files
for (const f of ['bad.txt', 'wrong-shape.json']) {
  if (!(await page.locator('[role="dialog"] input[type=file]').count())) { await page.getByRole('button', { name: 'Import', exact: true }).click(); await page.waitForTimeout(600); }
  await page.locator('[role="dialog"] input[type=file]').setInputFiles(DIR + f);
  await page.waitForTimeout(1800);
  console.log(`import ${f} -> toast:`, await toastText(), '| blocks:', await count());
}
await page.keyboard.press('Escape');
await page.waitForTimeout(500);
await app.done();
