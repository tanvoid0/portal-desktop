// QA-096 verify: a custom framework that IS mapped should still be editable/deletable from
// its row in the mappings list.
import { open } from './app.mjs';

const app = await open('settings/ides');
const page = app.page;

await page.getByRole('button', { name: 'Add Manually' }).click();
await page.waitForTimeout(500);
await page.getByRole('dialog').locator('input').first().fill('QA Editor');
const inputs = await page.getByRole('dialog').locator('input[type="text"], input:not([type])').all();
for (const inp of inputs.slice(1)) {
  const val = await inp.inputValue().catch(() => '');
  if (!val) await inp.fill('C:\\fake\\qa-editor.exe').catch(() => {});
}
await page.getByRole('dialog').getByRole('button', { name: /^Add IDE$/i }).click();
await page.waitForTimeout(1000);

await page.getByRole('button', { name: 'Create Framework' }).click();
await page.waitForTimeout(600);
await page.locator('#framework-name').fill('QA Framework');
await page.getByPlaceholder('e.g., logos:react, logos:vue').fill('logos:react');
await page.waitForTimeout(300);
await page.getByRole('dialog').getByRole('button', { name: /^Create Framework$/i }).click();
await page.waitForTimeout(1200);
await page.keyboard.press('Escape').catch(() => {});

const qaChip = page.getByRole('button', { name: 'QA Framework', exact: true });
await qaChip.scrollIntoViewIfNeeded();

// Before mapping: edit/delete overlay should be present (not the point of this issue, but
// confirms the fixture is in the expected starting state).
await qaChip.hover();
await page.waitForTimeout(300);
console.log('BEFORE mapping - chip overlay edit present:', await page.getByRole('button', { name: /edit qa framework/i }).count());

await qaChip.click();
await page.waitForTimeout(800);
const dialog = page.getByRole('dialog');
await dialog.getByRole('combobox').first().click();
await page.waitForTimeout(300);
await page.getByRole('option', { name: 'QA Editor', exact: true }).click();
await page.waitForTimeout(300);
await dialog.getByRole('button', { name: /Create Mapping|^Add$/i }).click();
await page.waitForTimeout(1200);

// --- The actual QA-096 test: now that QA Framework is mapped, is it still editable/deletable? ---
console.log('AFTER mapping - chip overlay edit present (old no-mapping branch, expect 0 since now in mapped list):', await page.getByRole('button', { name: /edit qa framework/i }).count());
const editFrameworkRowBtn = page.getByRole('button', { name: /edit qa framework/i });
const deleteFrameworkRowBtn = page.getByRole('button', { name: /delete qa framework/i });
console.log('AFTER mapping - row-level Edit QA Framework present:', await editFrameworkRowBtn.count());
console.log('AFTER mapping - row-level Delete QA Framework present:', await deleteFrameworkRowBtn.count());
await app.shot('QA-096-mapped-framework-still-editable');

if (await editFrameworkRowBtn.count()) {
  await editFrameworkRowBtn.first().click();
  await page.waitForTimeout(600);
  const editDialogTitle = await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]);
  console.log('Edit dialog opened:', editDialogTitle);
  await page.locator('#framework-name').fill('QA Framework edited');
  await page.getByRole('dialog').getByRole('button', { name: /^Update Framework$/i }).click();
  await page.waitForTimeout(1200);
  console.log('renamed chip present:', await page.getByRole('button', { name: 'QA Framework edited', exact: true }).count());
}

// --- Cleanup ---
const deleteMappingBtn = page.getByRole('button', { name: /delete mapping/i });
if (await deleteMappingBtn.count()) {
  await deleteMappingBtn.first().click();
  await page.waitForTimeout(500);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}
let delFrameworkBtn = page.getByRole('button', { name: /delete qa framework edited/i });
if (!(await delFrameworkBtn.count())) delFrameworkBtn = page.getByRole('button', { name: /delete qa framework/i });
if (await delFrameworkBtn.count()) {
  await delFrameworkBtn.first().click({ force: true });
  await page.waitForTimeout(500);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}
const deleteIdeBtn = page.getByRole('button', { name: /delete ide/i });
if (await deleteIdeBtn.count()) {
  await deleteIdeBtn.first().click();
  await page.waitForTimeout(500);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}
console.log('cleanup - QA Editor present:', await page.getByText('QA Editor', { exact: true }).count());
console.log('cleanup - QA Framework* present:', await page.evaluate(() => document.body.innerText.includes('QA Framework')));

await app.done();
