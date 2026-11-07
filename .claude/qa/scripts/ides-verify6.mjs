import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
// scroll to the mappings card and shoot it
await page.getByText('Add More Frameworks').scrollIntoViewIfNeeded();
await page.waitForTimeout(400);
await app.shot('ides-mappings-duplicate-chips');
const dup = await page.evaluate(() => {
  const h = [...document.querySelectorAll('p')].find(p => p.innerText.trim()==='Add More Frameworks');
  const names = [...h.parentElement.querySelectorAll('button')].map(b=>b.innerText.trim());
  const counts = {}; names.forEach(n=>counts[n]=(counts[n]||0)+1);
  return { total: names.length, dupes: Object.entries(counts).filter(([,c])=>c>1) };
});
console.log('Add More Frameworks chips:', JSON.stringify(dup));

// --- create a custom framework (DevIcon search)
console.log('=== create custom framework ===');
await page.getByRole('button', { name: 'Create Framework' }).click();
await page.waitForTimeout(600);
await page.locator('#framework-name').fill('QA Framework');
await page.locator('#framework-category').fill('QA');
await page.getByPlaceholder(/Search for icon/).fill('svelte');
await page.waitForTimeout(1200);
const results = await page.evaluate(() => [...document.querySelectorAll('[role="dialog"] .grid button')].map(b=>b.innerText.trim()));
console.log('devicon results:', JSON.stringify(results.slice(0,8)), 'count', results.length);
await app.shot('ides-framework-devicon-search');
if (results.length) await page.locator('[role="dialog"] .grid button').first().click();
await page.waitForTimeout(400);
await page.getByRole('dialog').getByRole('button', { name: 'Create Framework' }).click();
await page.waitForTimeout(1500);
console.log('toast:', await toastText());
console.log('chip exists:', await page.getByRole('button', { name: 'QA Framework', exact: true }).count());
await app.shot('ides-after-framework-created');
await app.done();
