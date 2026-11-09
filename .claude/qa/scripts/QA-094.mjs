// QA-094 verify, continuation: fixtures "QA Editor" + "QA Framework" already exist on the
// page. Create a mapping, test the mapping/framework/IDE delete confirm copy, then clean up
// all three fixtures.
import { open } from './app.mjs';

const app = await open('settings/ides');
const page = app.page;

async function confirmText() {
  return page.evaluate(() => document.querySelector('[role="alertdialog"]')?.innerText.replace(/\n/g, ' | '));
}

const qaChip = page.getByRole('button', { name: 'QA Framework', exact: true });
await qaChip.scrollIntoViewIfNeeded();
await qaChip.click();
await page.waitForTimeout(800);
console.log('dialog after clicking QA Framework chip:', await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n').slice(0, 2).join(' | ')));

const dialog = page.getByRole('dialog');
const ideCombo = dialog.getByRole('combobox').first();
if (await ideCombo.count()) {
  await ideCombo.click();
  await page.waitForTimeout(300);
  await page.getByRole('option', { name: 'QA Editor', exact: true }).click();
  await page.waitForTimeout(300);
}
await dialog.getByRole('button', { name: /Create Mapping|^Add$/i }).click();
await page.waitForTimeout(1200);
await page.keyboard.press('Escape').catch(() => {});
await app.shot('QA-094-after-mapping-created');

// --- Delete the mapping ---
const deleteMappingBtn = page.getByRole('button', { name: /delete mapping/i });
console.log('Delete mapping button present:', await deleteMappingBtn.count());
if (await deleteMappingBtn.count()) {
  await deleteMappingBtn.first().click();
  await page.waitForTimeout(600);
  console.log('mapping delete confirm dialog:', await confirmText());
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}

// --- Delete the framework ---
const delFrameworkBtn = page.getByRole('button', { name: /delete qa framework/i });
console.log('Delete framework button present:', await delFrameworkBtn.count());
if (await delFrameworkBtn.count()) {
  await delFrameworkBtn.first().click({ force: true });
  await page.waitForTimeout(600);
  console.log('framework delete confirm dialog:', await confirmText());
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}

// --- Delete the fixture IDE ---
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
await app.shot('QA-094-after-full-cleanup');
await app.done();
