import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const raw = await page.evaluate(() => [...document.querySelectorAll('div.rounded-md.border.p-4 p')].map(p => p.textContent));
console.log('path textContent:', JSON.stringify(raw));
console.log('mappings area:', await page.evaluate(() => {
  const b = document.body.innerText;
  const i = b.indexOf('Framework IDE Mappings');
  return b.slice(i, i+500).replace(/\n/g,' | ');
}));
// detected chip -> prefilled dialog
await page.getByRole('button', { name: 'Detect IDEs' }).first().click();
await page.waitForTimeout(6000);
const chip = page.getByRole('button', { name: /pycharm64\.exe/ }).first();
await chip.click();
await page.waitForTimeout(700);
console.log('dialog title:', await page.getByRole('dialog').locator('h2, [data-slot="dialog-title"]').first().innerText());
console.log('prefilled name:', await page.locator('#ide-name').inputValue());
console.log('prefilled exe:', JSON.stringify(await page.locator('#ide-executable').inputValue()));
await app.shot('ides-after-detected-chip-dialog');
await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();
await page.waitForTimeout(400);
await app.done();
