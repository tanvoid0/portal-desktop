// mappings: quick-add, edit, delete; custom framework create/edit/delete (DevIcon search).
import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText || '').replace(/\n/g,' | ').trim());
const mapBlock = () => page.evaluate(() => { const b=document.body.innerText; const i=b.lastIndexOf('Framework IDE Mappings'); return b.slice(i,i+400).replace(/\n/g,' | '); });

// --- quick-add a mapping (click the LEFT edge: the hover overlay covers the centre, see QA-092)
async function clickChip(name) {
  const c = page.getByRole('button', { name, exact: true }).first();
  await c.scrollIntoViewIfNeeded();
  const b = await c.boundingBox();
  await page.mouse.click(b.x + 10, b.y + b.height / 2);
  await page.waitForTimeout(800);
}
console.log('=== quick-add mapping from the Svelte chip ===');
await clickChip('Svelte');
console.log('dialog:', await page.evaluate(() => document.querySelector('[role="dialog"]')?.innerText.replace(/\n/g,' | ').slice(0,120)));
console.log('framework prefilled:', await page.locator('#mapping-framework').inputValue());
await page.getByRole('dialog').getByRole('combobox').click();
await page.waitForTimeout(400);
await page.getByRole('option').first().click();
await page.waitForTimeout(300);
await page.getByRole('dialog').getByRole('button', { name: 'Create Mapping' }).click();
await page.waitForTimeout(1500);
console.log('toast:', await toastText());
console.log('mappings:', await mapBlock());
await app.shot('ides-after-mapping-created');

console.log('=== reload, mapping persisted? ===');
await app.go('settings/ides');
console.log('mappings:', await mapBlock());

console.log('=== edit the mapping ===');
const mrow = page.locator('div').filter({ hasText: /^Svelte/ });
// find the mapping row buttons
const btns = await page.evaluate(() => {
  const rows = [...document.querySelectorAll('div.rounded-md.border, div.flex.items-center.justify-between')].filter(d => /Svelte/.test(d.innerText) && d.querySelectorAll('button').length >= 2 && d.innerText.length < 120);
  return rows.map(r => r.innerText.replace(/\n/g,' | '));
});
console.log('mapping rows found:', JSON.stringify(btns));
await app.done();
