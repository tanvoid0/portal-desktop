import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
await page.getByRole('button', { name: 'Add IDE' }).first().click();
await page.waitForTimeout(500);
await page.locator('#ide-name').fill('QA Path Test');
await page.locator('#ide-executable').fill('C:\Tools\QA\qa.exe');
console.log('typed into field:', JSON.stringify(await page.locator('#ide-executable').inputValue()));
await page.getByRole('dialog').getByRole('button', { name: 'Add IDE' }).click();
await page.waitForTimeout(1500);
console.log('rendered rows:', JSON.stringify(await page.evaluate(() => [...document.querySelectorAll('div.rounded-md.border.p-4')].map(d => ({ name: d.querySelector('span')?.textContent, path: d.querySelector('p')?.textContent })))));
// re-open edit on it, read what the field says
const rows = page.locator('div.rounded-md.border.p-4');
const n = await rows.count();
for (let i = 0; i < n; i++) {
  const t = await rows.nth(i).innerText();
  if (t.includes('QA Path Test')) {
    await rows.nth(i).locator('button').nth(1).click();  // edit (pencil)
    break;
  }
}
await page.waitForTimeout(700);
console.log('edit dialog exe field:', JSON.stringify(await page.locator('#ide-executable').inputValue()));
await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();
await app.done();
