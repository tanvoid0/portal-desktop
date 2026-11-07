import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());

// delete the custom framework
if (await page.getByRole('button', { name: 'QA Framework edited', exact: true }).count()) {
  await page.evaluate(() => { const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='QA Framework edited'); b.parentElement.querySelectorAll('div.absolute button')[1].click(); });
  await page.waitForTimeout(800);
  console.log('confirm:', await page.evaluate(()=>document.querySelector('[role="alertdialog"]')?.innerText.replace(/\n/g,' | ').slice(0,120)));
  await page.getByRole('alertdialog').getByRole('button').last().click();
  await page.waitForTimeout(1500);
  console.log('framework delete toast:', await toastText());
  console.log('chip gone:', (await page.getByRole('button', { name: 'QA Framework edited', exact: true }).count()) === 0);
}
// delete remaining QA IDEs
for (let i = 0; i < 5; i++) {
  const found = await page.evaluate(() => {
    const row = [...document.querySelectorAll('div.rounded-md.border.p-4')].find(d => /^QA /.test(d.innerText.trim()));
    if (!row) return null;
    const bs = row.querySelectorAll('button');
    bs[bs.length-1].click();
    return row.innerText.split('\n')[0];
  });
  if (!found) break;
  await page.waitForTimeout(800);
  await page.getByRole('alertdialog').getByRole('button').last().click();
  await page.waitForTimeout(1500);
  console.log('deleted IDE:', found);
}
await app.go('settings/ides');
console.log('remaining rows:', await page.evaluate(() => [...document.querySelectorAll('div.rounded-md.border.p-4')].map(d=>d.innerText.split('\n')[0])));
console.log('empty state back:', (await page.innerText('body')).includes('No IDEs configured'));
await app.shot('ides-after-cleanup-empty');
await app.done();
