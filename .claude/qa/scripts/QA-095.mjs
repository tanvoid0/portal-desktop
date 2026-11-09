// QA-095 verify: IDE row / mapping row / chip overlay edit+delete icon buttons should have
// accessible names. Creates fixtures, inspects accessible names, cleans up.
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

// IDE row buttons
const ideRowButtons = await page.evaluate(() => {
  const heading = [...document.querySelectorAll('*')].find((e) => e.textContent?.trim() === 'QA Editor' && e.tagName !== 'BODY');
  const row = heading?.closest('div.flex, div[class*="rounded"]');
  const btns = row ? [...row.querySelectorAll('button')] : [];
  return btns.map((b) => ({ text: b.textContent.trim(), label: b.getAttribute('aria-label'), title: b.getAttribute('title') }));
});
console.log('IDE row buttons:', JSON.stringify(ideRowButtons));

console.log('Edit IDE accessible button present:', await page.getByRole('button', { name: /edit ide/i }).count());
console.log('Delete IDE accessible button present:', await page.getByRole('button', { name: /delete ide/i }).count());

// Create framework + mapping to check mapping row + chip overlay names
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
await qaChip.hover();
await page.waitForTimeout(300);
console.log('chip overlay Edit QA Framework present:', await page.getByRole('button', { name: /edit qa framework/i }).count());
console.log('chip overlay Delete QA Framework present:', await page.getByRole('button', { name: /delete qa framework/i }).count());

await qaChip.click();
await page.waitForTimeout(800);
const dialog = page.getByRole('dialog');
const ideCombo = dialog.getByRole('combobox').first();
await ideCombo.click();
await page.waitForTimeout(300);
await page.getByRole('option', { name: 'QA Editor', exact: true }).click();
await page.waitForTimeout(300);
await dialog.getByRole('button', { name: /Create Mapping|^Add$/i }).click();
await page.waitForTimeout(1200);

console.log('mapping row Edit mapping present:', await page.getByRole('button', { name: /edit mapping/i }).count());
console.log('mapping row Delete mapping present:', await page.getByRole('button', { name: /delete mapping/i }).count());
await app.shot('QA-095-mapping-row-names');

// --- Cleanup ---
const deleteMappingBtn = page.getByRole('button', { name: /delete mapping/i });
if (await deleteMappingBtn.count()) {
  await deleteMappingBtn.first().click();
  await page.waitForTimeout(500);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}
const delFrameworkBtn = page.getByRole('button', { name: /delete qa framework/i });
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
console.log('cleanup - QA Framework present:', await page.getByRole('button', { name: 'QA Framework', exact: true }).count());

await app.done();
