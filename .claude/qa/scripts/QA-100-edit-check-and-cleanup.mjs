import { open } from './app.mjs';

const app = await open('automation/blocks');
const page = app.page;
await page.waitForTimeout(1500);
await page.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page.waitForTimeout(1000);

// Try editing the fixture template's name, see if it actually persists.
const card = page.locator('[data-slot="card"]').filter({ hasText: 'QA Imported Template' }).first();
const editBtn = card.getByRole('button', { name: /edit template/i });
console.log('edit button present:', await editBtn.count());
if (await editBtn.count()) {
  await editBtn.first().click();
  await page.waitForTimeout(700);
  const dialogTitle = await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]);
  console.log('edit dialog:', dialogTitle);
  const nameInput = page.getByRole('dialog').locator('input').first();
  await nameInput.fill('QA Imported Template EDITED');
  await page.waitForTimeout(300);
  const saveBtn = page.getByRole('dialog').getByRole('button', { name: /^Save|^Update/i });
  if (await saveBtn.count()) {
    await saveBtn.first().click();
    await page.waitForTimeout(1000);
  } else {
    console.log('no save button found, dialog buttons:', await page.getByRole('dialog').locator('button').allTextContents());
  }
  console.log('renamed card visible:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template EDITED')));
  console.log('old name still visible:', await page.evaluate(() => document.body.innerText.includes('QA Imported Template') && !document.body.innerText.includes('EDITED')));
}
await app.shot('QA-100-edit-attempt-result');

// --- Direct localStorage cleanup, since in-app delete is confirmed broken ---
const removed = await page.evaluate(() => {
  const KEY = 'portal.pipelines.customTemplates';
  const raw = localStorage.getItem(KEY);
  if (!raw) return 'no storage key found';
  const arr = JSON.parse(raw);
  const filtered = arr.filter((t) => !String(t.name || '').startsWith('QA Imported Template'));
  localStorage.setItem(KEY, JSON.stringify(filtered));
  return `removed ${arr.length - filtered.length} of ${arr.length}`;
});
console.log('localStorage cleanup:', removed);

await app.done();

// Verify with a fresh load
const app2 = await open('automation/blocks');
const page2 = app2.page;
await page2.waitForTimeout(1500);
await page2.evaluate(() => {
  const b = [...document.querySelectorAll('button,[role="tab"]')].find((x) => x.textContent.trim() === 'Templates');
  b?.click();
});
await page2.waitForTimeout(1000);
console.log('present after cleanup + fresh reload:', await page2.evaluate(() => document.body.innerText.includes('QA Imported Template')));
await app2.done();
