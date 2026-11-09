// QA-103 verify: every framework chip should appear exactly once in "Add More Frameworks",
// in both the no-mapping and has-mapping branches.
import { open } from './app.mjs';

const app = await open('settings/ides');
const page = app.page;

async function countChips() {
  return page.evaluate(() => {
    // All chip buttons sit after the "Framework IDE Mappings" card; grab every button whose
    // only content is an icon + plain text (chips), skipping the icon-only edit/delete ones.
    const buttons = [...document.querySelectorAll('button')];
    const names = buttons
      .map((b) => b.textContent.trim())
      .filter((t) => t && t.length > 0 && !/^(Edit|Delete|Set Default|Detect|Refresh|Add|Create)/.test(t));
    const counts = {};
    for (const n of names) counts[n] = (counts[n] || 0) + 1;
    return { total: names.length, distinct: Object.keys(counts).length, dupes: Object.entries(counts).filter(([, c]) => c > 1) };
  });
}

console.log('--- no IDE / no mappings state ---');
console.log(JSON.stringify(await countChips()));

// Add an IDE (no mappings yet) - this is the "no mappings" branch with the full chip list.
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

console.log('--- IDE present, no mappings yet (no-mappings branch) ---');
console.log(JSON.stringify(await countChips()));
await app.shot('QA-103-no-mappings-branch');

// Create one mapping so the has-mappings branch (unmappedChipsByCategory) renders instead.
const reactChip = page.getByRole('button', { name: 'React', exact: true }).first();
await reactChip.scrollIntoViewIfNeeded();
// Click the left portion of the chip to avoid the overlay (QA-102).
const box = await reactChip.boundingBox();
await page.mouse.click(box.x + 8, box.y + box.height / 2);
await page.waitForTimeout(700);
const dialog = page.getByRole('dialog');
if (await dialog.count()) {
  const combo = dialog.getByRole('combobox').first();
  if (await combo.count()) {
    await combo.click();
    await page.waitForTimeout(300);
    await page.getByRole('option', { name: 'QA Editor', exact: true }).click();
    await page.waitForTimeout(300);
  }
  await dialog.getByRole('button', { name: /Create Mapping|^Add$/i }).click();
  await page.waitForTimeout(1200);
}

console.log('--- has-mappings branch (1 mapping present) ---');
console.log(JSON.stringify(await countChips()));
await app.shot('QA-103-has-mappings-branch');

// --- Cleanup ---
const deleteMappingBtn = page.getByRole('button', { name: /delete mapping/i });
if (await deleteMappingBtn.count()) {
  await deleteMappingBtn.first().click();
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
await app.done();
