import { open } from './app.mjs';

const app = await open('automation/blocks');
const page = app.page;
await page.waitForTimeout(1500);
await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);

const card = page.locator('[data-slot="card"]').filter({ hasText: 'QA Imported Template' }).first();
console.log('card present:', await card.count());
const deleteBtn = card.getByRole('button', { name: /delete template qa imported template/i });
console.log('delete button present:', await deleteBtn.count());
await deleteBtn.first().click();
await page.waitForTimeout(800);
const confirmDialogText = await page.evaluate(() => document.querySelector('[role="alertdialog"]')?.innerText.replace(/\n/g, ' | '));
console.log('confirm dialog:', confirmDialogText);
const confirmBtn = page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i });
console.log('confirm Delete button present:', await confirmBtn.count());
if (await confirmBtn.count()) {
  await confirmBtn.click();
} else {
  // fallback: last button in the dialog
  await page.getByRole('alertdialog').getByRole('button').last().click();
}
await page.waitForTimeout(1200);
console.log('present after delete attempt:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app.shot('QA-100-cleanup-result');
await app.done();
