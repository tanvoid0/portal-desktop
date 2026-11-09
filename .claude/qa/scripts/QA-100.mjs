// QA-100 verify: an imported template should persist across reload and be editable/deletable.
import { open } from './app.mjs';

const FILE = 'C:/Users/tanvo/AppData/Local/Temp/claude/7b9877c8f6cf97d9/11d8b169-8f1b-4130-bc7e-2b0e072046e3/scratchpad/qa-template-valid.json';

let app = await open('automation/blocks');
let page = app.page;
await page.waitForTimeout(1500);

await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);

const count = () => page.evaluate(() => document.querySelectorAll('[data-slot="card"]').length);
console.log('templates before import:', await count());

await page.getByRole('button', { name: /Import Template/ }).first().click();
await page.waitForTimeout(700);
await page.locator('input[type=file]').last().setInputFiles(FILE);
await page.waitForTimeout(2000);
const toastText = await page.evaluate(() => document.querySelector('[aria-label="Notifications"]')?.innerText);
console.log('import toast:', toastText);
console.log('templates after import:', await count());
console.log('present after import:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app.shot('QA-100-after-import');
await app.done();

// --- Reload and check persistence ---
app = await open('automation/blocks');
page = app.page;
await page.waitForTimeout(1500);
await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);
console.log('AFTER RELOAD - present:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
console.log('AFTER RELOAD - templates count:', await count());
await app.shot('QA-100-after-reload');

// --- Edit/Delete buttons present? ---
const card = page.locator('[data-slot="card"]').filter({ hasText: 'QA Imported Template' }).first();
const buttons = await card.evaluate((c) => [...c.querySelectorAll('button')].map((b) => b.getAttribute('aria-label') || b.textContent.trim() || '(icon, no label)'));
console.log('card buttons:', JSON.stringify(buttons));

// --- Cleanup: delete the fixture template ---
const deleteBtn = card.getByRole('button', { name: /delete/i });
if (await deleteBtn.count()) {
  await deleteBtn.first().click();
  await page.waitForTimeout(600);
  const confirmBtn = page.getByRole('alertdialog').getByRole('button').last();
  if (await confirmBtn.count()) {
    await confirmBtn.click();
    await page.waitForTimeout(1000);
  }
}
console.log('AFTER CLEANUP - present:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app.done();
