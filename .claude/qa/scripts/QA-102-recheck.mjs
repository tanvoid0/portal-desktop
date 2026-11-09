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

const reactChip = page.getByRole('button', { name: 'React', exact: true }).first();
await reactChip.scrollIntoViewIfNeeded();
const box = await reactChip.boundingBox();
const cx = box.x + box.width / 2;
const cy = box.y + box.height / 2;

const geom = await page.evaluate(({ cx, cy }) => {
  const el = document.elementFromPoint(cx, cy);
  return {
    tag: el?.tagName,
    label: el?.getAttribute('aria-label'),
    text: el?.textContent?.trim().slice(0, 30),
    isCustomMarkerNearby: el?.closest('.group')?.querySelector('[aria-label*="Edit"]')?.getAttribute('aria-label'),
  };
}, { cx, cy });
console.log('elementFromPoint(chip center):', JSON.stringify(geom));

await page.mouse.click(cx, cy);
await page.waitForTimeout(700);
const dialogTitle = await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]);
console.log('dialog after click:', dialogTitle);
await app.shot('QA-102-recheck-react-chip-click');
await page.keyboard.press('Escape').catch(() => {});

// Is "React" really isCustom in this data (why does it have the overlay)?
const chipHtml = await reactChip.evaluate((el) => el.parentElement.outerHTML.slice(0, 600));
console.log('React chip wrapper HTML:', chipHtml);

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
