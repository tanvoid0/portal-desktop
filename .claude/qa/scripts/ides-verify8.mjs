import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const mapBlock = () => page.evaluate(() => { const b=document.body.innerText; const i=b.lastIndexOf('Framework IDE Mappings'); return b.slice(i,i+120).replace(/\n/g,' | '); });

console.log('with a mapping present, does the custom framework chip still carry edit/delete?',
  await page.evaluate(() => {
    const b = [...document.querySelectorAll('button')].filter(x => x.innerText.trim() === 'QA Framework');
    return b.map(x => !!x.parentElement.querySelector('div.absolute'));
  }));

console.log('=== edit the mapping ===');
await page.evaluate(() => {
  const row = [...document.querySelectorAll('div.rounded-md.border.p-4')].find(d => /Svelte/.test(d.innerText));
  row.querySelectorAll('button')[0].click();
});
await page.waitForTimeout(800);
console.log('dialog:', await page.evaluate(()=>document.querySelector('[role="dialog"]')?.innerText.split('\n')[0]));
console.log('framework field:', await page.locator('#mapping-framework').inputValue());
await page.locator('#mapping-framework').fill('SvelteKit');
await page.getByRole('dialog').getByRole('button', { name: 'Update Mapping' }).click();
await page.waitForTimeout(1500);
console.log('toast:', await toastText(), '| mappings:', await mapBlock());

console.log('=== delete the mapping ===');
await page.evaluate(() => {
  const row = [...document.querySelectorAll('div.rounded-md.border.p-4')].find(d => /SvelteKit/.test(d.innerText));
  row.querySelectorAll('button')[1].click();
});
await page.waitForTimeout(900);
console.log('confirm:', await page.evaluate(()=>document.querySelector('[role="alertdialog"]')?.innerText.replace(/\n/g,' | ').slice(0,150)));
await page.getByRole('alertdialog').getByRole('button').last().click();
await page.waitForTimeout(1500);
console.log('toast:', await toastText(), '| mappings:', await mapBlock());
await app.shot('ides-mappings-empty-after-delete');

console.log('=== now the overlay is back ===');
console.log(await page.evaluate(() => {
  const b = [...document.querySelectorAll('button')].filter(x => x.innerText.trim() === 'QA Framework');
  return b.map(x => !!x.parentElement.querySelector('div.absolute'));
}));
await app.done();
