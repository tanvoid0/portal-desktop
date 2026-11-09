// QA-102 round-2 re-verify: dead-center click on an UNHOVERED chip must not hit the overlay.
// Playwright's mouse.click() physically moves the cursor there first, which itself satisfies
// CSS :hover before the click fires -- not a fair test of the "invisible AND unhovered" case.
// Use a JS-dispatched click (no real mouse movement, no :hover) to isolate that, then do a
// real hover+click on the actual edit icon to confirm the overlay still works when intended.
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

// 1. computed pointer-events on the overlay without ever hovering it.
const unhoveredStyle = await page.evaluate(() => {
  const chip = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'React');
  const wrapper = chip.parentElement;
  const overlay = wrapper.querySelector('.absolute, [class*="opacity-0"]');
  const editBtn = wrapper.querySelector('[aria-label*="Edit"]');
  return {
    overlayPointerEvents: overlay ? getComputedStyle(overlay).pointerEvents : null,
    editBtnPointerEvents: editBtn ? getComputedStyle(editBtn).pointerEvents : null,
  };
});
console.log('UNHOVERED computed pointer-events:', JSON.stringify(unhoveredStyle));

// 2. JS-dispatched click at chip center -- no real mouse movement, no :hover triggered.
const dialogBefore = await page.evaluate(() => !!document.querySelector('[role="dialog"]'));
const clickResult = await page.evaluate(() => {
  const chip = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'React');
  const rect = chip.getBoundingClientRect();
  const cx = rect.left + rect.width / 2;
  const cy = rect.top + rect.height / 2;
  const el = document.elementFromPoint(cx, cy);
  el.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, clientX: cx, clientY: cy }));
  return { clickedTag: el.tagName, clickedLabel: el.getAttribute('aria-label'), clickedText: el.textContent.trim() };
});
console.log('JS-dispatched click (no hover) target:', JSON.stringify(clickResult));
await page.waitForTimeout(600);
const dialogTitleNoHover = await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]);
console.log('dialog after unhovered JS click:', dialogTitleNoHover, '(expect "Add Framework IDE Mapping")');
await page.keyboard.press('Escape').catch(() => {});
await page.waitForTimeout(400);

// 3. Real hover + click directly on the edit icon -- should still work when intended.
await reactChip.hover();
await page.waitForTimeout(300);
const hoveredStyle = await page.evaluate(() => {
  const chip = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'React');
  const editBtn = chip.parentElement.querySelector('[aria-label*="Edit"]');
  return editBtn ? getComputedStyle(editBtn).pointerEvents : null;
});
console.log('HOVERED editBtn pointer-events (expect auto):', hoveredStyle);
const editBtn = page.getByRole('button', { name: /edit react/i });
if (await editBtn.count()) {
  await editBtn.first().click();
  await page.waitForTimeout(600);
  console.log('dialog after intentional edit-icon click:', await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]));
  await page.keyboard.press('Escape').catch(() => {});
}
await app.shot('QA-102-recheck2-result');

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
