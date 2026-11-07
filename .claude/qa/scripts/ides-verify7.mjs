import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const mapBlock = () => page.evaluate(() => { const b=document.body.innerText; const i=b.lastIndexOf('Framework IDE Mappings'); return b.slice(i,i+160).replace(/\n/g,' | '); });

// edit the custom framework via its (invisible) overlay pencil
const chip = page.getByRole('button', { name: 'QA Framework', exact: true }).first();
await chip.scrollIntoViewIfNeeded();
await page.evaluate(() => {
  const btn = [...document.querySelectorAll('button')].find(x => x.innerText.trim() === 'QA Framework');
  btn.parentElement.querySelector('div.absolute button').click();   // pencil
});
await page.waitForTimeout(700);
console.log('dialog:', await page.evaluate(()=>document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]));
await page.locator('#framework-name').fill('QA Framework edited');
await page.getByRole('dialog').getByRole('button', { name: 'Update Framework' }).click();
await page.waitForTimeout(1500);
console.log('edit toast:', await toastText());
console.log('renamed chip present:', await page.getByRole('button', { name: 'QA Framework edited', exact: true }).count());

// file-icon branch: open the dialog and check the File tab renders (do NOT open the native picker)
await page.getByRole('button', { name: 'Create Framework' }).click();
await page.waitForTimeout(600);
await page.getByRole('dialog').getByRole('button', { name: 'File', exact: true }).click();
await page.waitForTimeout(400);
console.log('file branch shows picker button:', await page.getByRole('button', { name: 'Select Icon File' }).count());
await app.shot('ides-framework-file-icon-branch');
await page.getByRole('dialog').getByRole('button', { name: 'Cancel' }).click();
await page.waitForTimeout(400);

// delete the custom framework
const c2 = page.getByRole('button', { name: 'QA Framework edited', exact: true }).first();
await c2.scrollIntoViewIfNeeded();
await page.evaluate(() => {
  const btn = [...document.querySelectorAll('button')].find(x => x.innerText.trim() === 'QA Framework edited');
  btn.parentElement.querySelectorAll('div.absolute button')[1].click();   // X
});
await page.waitForTimeout(800);
console.log('confirm dialog:', await page.evaluate(()=>document.querySelector('[role="alertdialog"],[role="dialog"]')?.innerText.replace(/\n/g,' | ').slice(0,160)));
const confirmBtn = page.getByRole('button', { name: /^(Delete|Confirm|Continue)/ }).last();
if (await confirmBtn.count()) await confirmBtn.click();
await page.waitForTimeout(1500);
console.log('delete toast:', await toastText());
console.log('chip gone:', (await page.getByRole('button', { name: 'QA Framework edited', exact: true }).count()) === 0);
await app.done();
