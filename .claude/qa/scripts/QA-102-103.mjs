// QA-102 verify: clicking the center of a framework chip should open the "Add mapping"
// dialog, not "Edit Framework" (the invisible overlay no longer swallows the click).
// QA-103 verify: no duplicate chip names in "Add More Frameworks".
import { open } from './app.mjs';

const app = await open('settings/ides');
const page = app.page;

// Need at least one IDE for the chip grid to render at all.
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

// --- QA-103: count chip names ---
const chipNames = await page.evaluate(() => {
  const heading = [...document.querySelectorAll('*')].find((e) => e.textContent.trim() === 'Add More Frameworks' || e.textContent.trim() === 'Frameworks');
  if (!heading) return null;
  const container = heading.closest('div')?.parentElement || heading.parentElement;
  const btns = container ? [...container.querySelectorAll('button')] : [];
  return btns.map((b) => b.textContent.trim()).filter(Boolean);
});
if (chipNames) {
  const counts = {};
  for (const n of chipNames) counts[n] = (counts[n] || 0) + 1;
  const dupes = Object.entries(counts).filter(([, c]) => c > 1);
  console.log('total chips:', chipNames.length, 'distinct:', Object.keys(counts).length, 'dupes:', JSON.stringify(dupes));
} else {
  console.log('could not locate chip container');
}

// --- QA-102: click center of the React chip ---
const reactChip = page.getByRole('button', { name: 'React', exact: true }).first();
await reactChip.scrollIntoViewIfNeeded();
const box = await reactChip.boundingBox();
console.log('React chip box:', JSON.stringify(box));
await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
await page.waitForTimeout(700);
const dialogTitle = await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]);
console.log('dialog opened after clicking chip center:', dialogTitle, '(expect "Add Framework IDE Mapping")');
await app.shot('QA-102-after-chip-center-click');
await page.keyboard.press('Escape').catch(() => {});
await page.waitForTimeout(400);

// --- Cleanup ---
const deleteIdeBtn = page.getByRole('button', { name: /delete ide/i });
if (await deleteIdeBtn.count()) {
  await deleteIdeBtn.first().click();
  await page.waitForTimeout(500);
  await page.getByRole('alertdialog').getByRole('button', { name: /^Delete$/i }).click();
  await page.waitForTimeout(1000);
}
console.log('cleanup - QA Editor present:', await page.getByText('QA Editor', { exact: true }).count());

await app.done();
