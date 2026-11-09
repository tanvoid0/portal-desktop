// QA-100 round-2 re-verify: import a framework-less template, edit it, then delete it.
import { open } from './app.mjs';

const FILE = 'C:/Users/tanvo/AppData/Local/Temp/claude/7b9877c8f6cf97d9/11d8b169-8f1b-4130-bc7e-2b0e072046e3/scratchpad/qa-template-valid.json';

const app = await open('automation/blocks');
const page = app.page;
await page.waitForTimeout(1500);
await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);

await page.getByRole('button', { name: /Import Template/ }).first().click();
await page.waitForTimeout(700);
await page.locator('input[type=file]').last().setInputFiles(FILE);
await page.waitForTimeout(1500);
console.log('import present:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));

const card = page.locator('[data-slot="card"]').filter({ hasText: 'QA Imported Template' }).first();
const editBtn = card.getByRole('button', { name: /edit template/i });
await editBtn.first().click();
await page.waitForTimeout(700);
const nameInput = page.getByRole('dialog').locator('input').first();
await nameInput.fill('QA Imported Template EDITED');
await page.waitForTimeout(300);
const saveBtn = page.getByRole('dialog').getByRole('button', { name: /^Save|^Update/i });
await saveBtn.first().click();
await page.waitForTimeout(1000);
const errToast = await page.evaluate(() => document.querySelector('[aria-label="Notifications"]')?.innerText);
console.log('toast after edit save:', errToast);
console.log('renamed card visible:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template EDITED')));
await app.shot('QA-100-edit-recheck-result');

// --- Delete the (possibly renamed) fixture ---
const nameNow = (await page.evaluate(() => document.body.innerText.includes('QA Imported Template EDITED')))
  ? 'QA Imported Template EDITED'
  : 'QA Imported Template';
const card2 = page.locator('[data-slot="card"]').filter({ hasText: nameNow }).first();
const deleteBtn = card2.getByRole('button', { name: /delete template/i });
console.log('delete button present:', await deleteBtn.count());
if (await deleteBtn.count()) {
  await deleteBtn.first().click();
  await page.waitForTimeout(600);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1200);
}
console.log('present after delete:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app.done();

// Verify with a hard reload
const app2 = await open('automation/blocks');
const page2 = app2.page;
await page2.waitForTimeout(1500);
await page2.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page2.waitForTimeout(1000);
console.log('present after fresh reload:', await page2.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app2.done();
