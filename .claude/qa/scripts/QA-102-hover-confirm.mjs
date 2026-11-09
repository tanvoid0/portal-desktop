import { open } from './app.mjs';
const app = await open('settings/ides');
const page = app.page;

// QA Editor fixture already exists from the previous (crashed) script; reuse it.
const reactChip = page.getByRole('button', { name: 'React', exact: true }).first();
await reactChip.scrollIntoViewIfNeeded();
const box = await reactChip.boundingBox();

// Hover the chip (mouse.move triggers the real :hover/group-hover state), wait for the
// opacity transition to finish, then click precisely on the overlay's icon position
// (top-right corner) with force to skip Playwright's actionability recheck.
await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
await page.waitForTimeout(400); // let the opacity transition finish
const editX = box.x + box.width - 10; // near the right edge, inside the edit icon
const editY = box.y + 4;
await page.mouse.click(editX, editY);
await page.waitForTimeout(700);
console.log('dialog after hovering then clicking the edit icon:', await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]));
await page.keyboard.press('Escape').catch(() => {});
await page.waitForTimeout(400);

// cleanup
const deleteIdeBtn = page.getByRole('button', { name: /delete ide/i });
if (await deleteIdeBtn.count()) {
  await deleteIdeBtn.first().click();
  await page.waitForTimeout(500);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}
console.log('cleanup - QA Editor present:', await page.getByText('QA Editor', { exact: true }).count());
await app.done();
