import { open } from './app.mjs';

const app = await open('settings/ides');
const page = app.page;

async function confirmText() {
  return page.evaluate(() => document.querySelector('[role="alertdialog"]')?.innerText.replace(/\n/g, ' | '));
}

// --- Add fixture IDE (kept this time, for mapping) ---
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
console.log('QA Editor added:', await page.getByText('QA Editor', { exact: true }).count());

// --- Create fixture Framework ---
await page.getByRole('button', { name: 'Create Framework' }).click();
await page.waitForTimeout(600);
await page.locator('#framework-name').fill('QA Framework');
await page.getByPlaceholder('e.g., logos:react, logos:vue').fill('logos:react');
await page.waitForTimeout(300);
await page.getByRole('dialog').getByRole('button', { name: /^Create Framework$/i }).click();
await page.waitForTimeout(1200);
console.log('QA Framework created:', await page.getByRole('button', { name: 'QA Framework', exact: true }).count());
await app.shot('QA-094b-after-fixtures-created');

// --- Create a mapping between QA Framework and QA Editor ---
await page.getByRole('button', { name: 'Add Mapping' }).click();
await page.waitForTimeout(600);
const dialog = page.getByRole('dialog');
const combos = dialog.getByRole('combobox');
const comboCount = await combos.count();
console.log('Add Mapping dialog combobox count:', comboCount);
if (comboCount >= 1) {
  await combos.first().click();
  await page.waitForTimeout(300);
  await page.getByRole('option', { name: 'QA Framework', exact: true }).click();
  await page.waitForTimeout(300);
}
if (comboCount >= 2) {
  await combos.nth(1).click();
  await page.waitForTimeout(300);
  await page.getByRole('option', { name: 'QA Editor', exact: true }).click();
  await page.waitForTimeout(300);
}
await page.getByRole('dialog').getByRole('button', { name: /Create Mapping|^Add$/i }).click();
await page.waitForTimeout(1200);
await app.shot('QA-094b-after-mapping-created');

// --- Delete the mapping, check confirm copy ---
const deleteMappingBtn = page.getByRole('button', { name: /delete mapping/i });
console.log('Delete mapping button present:', await deleteMappingBtn.count());
if (await deleteMappingBtn.count()) {
  await deleteMappingBtn.first().click();
  await page.waitForTimeout(600);
  console.log('mapping delete confirm dialog:', await confirmText());
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}

// --- Delete the framework, check confirm copy ---
const delFrameworkBtn = page.getByRole('button', { name: /delete qa framework/i });
console.log('Delete framework button present:', await delFrameworkBtn.count());
if (await delFrameworkBtn.count()) {
  await delFrameworkBtn.first().click({ force: true });
  await page.waitForTimeout(600);
  console.log('framework delete confirm dialog:', await confirmText());
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}

// --- Cleanup: delete the fixture IDE ---
const deleteIdeBtn = page.getByRole('button', { name: /delete ide/i });
if (await deleteIdeBtn.count()) {
  await deleteIdeBtn.first().click();
  await page.waitForTimeout(600);
  console.log('ide delete confirm dialog:', await confirmText());
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}

console.log('final QA Editor present:', await page.getByText('QA Editor', { exact: true }).count());
console.log('final QA Framework present:', await page.getByRole('button', { name: 'QA Framework', exact: true }).count());
await app.shot('QA-094b-after-full-cleanup');
await app.done();
