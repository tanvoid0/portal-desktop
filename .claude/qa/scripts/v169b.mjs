import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('scripts/new');
const p = app.page;
await dismissVault(p);
await p.getByPlaceholder('e.g., VPN Connect').fill('QA script long ' + 'x'.repeat(90));
await p.getByPlaceholder('Describe what this script does').fill('QA fixture');
await p.getByPlaceholder(/e.g., \.\/scripts/).fill('echo qa');
await p.getByRole('button', { name: 'Save Script' }).click();
await p.waitForTimeout(1500);
const wide = () => p.evaluate(() => [...document.querySelectorAll('*')].filter(e => e.scrollWidth > e.clientWidth + 1 && e.clientWidth>0 && getComputedStyle(e).overflowX!=='hidden' && ['DIV','MAIN'].includes(e.tagName)).map(e=>e.scrollWidth+'>'+e.clientWidth).slice(0,4));
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({ width: w, height: h });
  for (const r of ['automation/scripts','automation/utilities']) {
    await p.goto(new URL('/'+r, p.url()).href); await dismissVault(p,1500); await p.waitForTimeout(1500);
    await p.getByPlaceholder('Search scripts...').fill('QA script long'); await p.waitForTimeout(600);
    console.log(r, w, 'overflow:', await wide(), await app.shot(`QA-169-after-${r.replace('/','-')}-${w}`));
  }
}
await p.setViewportSize({ width: 1280, height: 800 });
await p.getByRole('button', { name: 'View' }).first().click(); await p.waitForTimeout(1200);
console.log('detail overflow:', await wide());
await p.goBack(); await dismissVault(p,800);
await p.getByPlaceholder('Search scripts...').fill('QA script long'); await p.waitForTimeout(500);
for (let i = 0; i < 5; i++) { const d = p.getByRole('button', { name: /^Delete script QA script long/ }); if (!(await d.count())) break; await d.first().click(); await p.waitForTimeout(500); await p.locator('[role=alertdialog],[role=dialog]').getByRole('button', { name: 'Delete' }).click(); await p.waitForTimeout(800); }
await app.done();
