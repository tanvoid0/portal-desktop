import { open } from './i2.mjs';
const app = await open('automation/blocks', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(2500);
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const del = page.getByRole('button', { name: /Delete block QA Imported Block/ });
console.log('fixture delete buttons:', await del.count());
while (await del.count()) {
  await del.first().click();
  await page.waitForTimeout(800);
  const ad = page.getByRole('alertdialog');
  if (await ad.count()) { console.log('confirm:', (await ad.innerText()).replace(/\n/g,' | ').slice(0,120)); await ad.getByRole('button').last().click(); }
  await page.waitForTimeout(1500);
}
console.log('toast:', await toastText());
await app.go('automation/blocks');
console.log('QA Imported Block still present:', (await page.innerText('body')).includes('QA Imported Block'));
console.log('blocks:', await page.evaluate(()=>[...document.querySelectorAll('[data-slot="card"]')].filter(c=>c.getBoundingClientRect().y>150).length));
await app.done();
