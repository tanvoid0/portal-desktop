import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const mapBlock = () => page.evaluate(() => { const b=document.body.innerText; const i=b.lastIndexOf('Framework IDE Mappings'); return b.slice(i,i+110).replace(/\n/g,' | '); });

console.log('=== framework edit (overlay pencil) ===');
await page.evaluate(() => { const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='QA Framework'); b.parentElement.querySelector('div.absolute button').click(); });
await page.waitForTimeout(800);
console.log('dialog:', await page.evaluate(()=>document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]));
await page.locator('#framework-name').fill('QA Framework edited');
await page.getByRole('dialog').getByRole('button', { name: 'Update Framework' }).click();
await page.waitForTimeout(1500);
console.log('toast:', await toastText(), '| renamed chip:', await page.getByRole('button', { name: 'QA Framework edited', exact: true }).count());

console.log('=== file-icon branch ===');
await page.getByRole('button', { name: 'Create Framework' }).click();
await page.waitForTimeout(600);
await page.getByRole('dialog').getByRole('button', { name: 'File', exact: true }).click();
await page.waitForTimeout(400);
console.log('Select Icon File button:', await page.getByRole('button', { name: 'Select Icon File' }).count());
await app.shot('ides-framework-file-icon-branch');
await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();
await page.waitForTimeout(500);

console.log('=== mapping on QA Framework edited, then delete the IDE ===');
const c = page.getByRole('button', { name: 'QA Framework edited', exact: true }).first();
await c.scrollIntoViewIfNeeded();
const bb = await c.boundingBox();
await page.mouse.click(bb.x + 10, bb.y + bb.height/2);
await page.waitForTimeout(800);
await page.getByRole('dialog').getByRole('combobox').click();
await page.waitForTimeout(400);
await page.getByRole('option').first().click();
await page.getByRole('dialog').getByRole('button', { name: 'Create Mapping' }).click();
await page.waitForTimeout(1500);
console.log('mappings:', await mapBlock());
const ideName = await page.evaluate(() => { const b=document.body.innerText; const m=b.match(/QA Framework edited\s*\n?\s*\n?(QA [^\n]*)/); return m?m[1]:null; });
console.log('mapped to:', ideName);

await page.evaluate(() => {
  const row = [...document.querySelectorAll('div.rounded-md.border.p-4')].find(d => /QA Editor/.test(d.innerText) && /Default/.test(d.innerText));
  row.querySelectorAll('button')[row.querySelectorAll('button').length-1].click();
});
await page.waitForTimeout(900);
console.log('confirm:', await page.evaluate(()=>document.querySelector('[role="alertdialog"]')?.innerText.replace(/\n/g,' | ').slice(0,130)));
await page.getByRole('alertdialog').getByRole('button').last().click();
await page.waitForTimeout(2000);
console.log('toast:', await toastText());
console.log('mappings after deleting the IDE:', await mapBlock());
await app.shot('ides-after-ide-delete-with-mapping');
await app.done();
