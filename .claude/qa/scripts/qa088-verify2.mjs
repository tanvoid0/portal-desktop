import { open } from './i2.mjs';
const DIR = 'C:/Users/tanvo/AppData/Local/Temp/claude/660cd8302a863ca1/2609a446-530c-4f61-83da-a545cae02459/scratchpad/qafiles/';
const app = await open('automation/blocks', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(2500);
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const count = () => page.evaluate(() => [...document.querySelectorAll('[data-slot="card"]')].filter(c=>c.getBoundingClientRect().y>150).length);

// --- category filters + search on Blocks
for (const cat of ['Build','Test','Deploy','Utility','All']) {
  await page.getByRole('button', { name: cat, exact: true }).click();
  await page.waitForTimeout(500);
  console.log(`filter ${cat}: ${await count()} cards`);
}
await page.getByPlaceholder('Search blocks...').fill('pytest');
await page.waitForTimeout(600);
console.log('search "pytest":', await count(), 'cards');
await page.getByPlaceholder('Search blocks...').fill('zzzznope');
await page.waitForTimeout(600);
console.log('search "zzzznope":', await count(), '| empty text:', await page.evaluate(()=>{const b=document.body.innerText; const i=b.indexOf('No blocks'); return i<0?'(no "No blocks" text)':b.slice(i,i+160).replace(/\n/g,' | ');}));
await app.shot('UX-007-after-blocks-search-empty');

// --- switch to Templates WITH the query still typed
await page.evaluate(()=>{const b=[...document.querySelectorAll('button,[role="tab"]')].find(x=>x.innerText.trim()==='Templates'); b.click();});
await page.waitForTimeout(1200);
console.log('Templates tab with query still set -> empty state:', await page.evaluate(()=>{const b=document.body.innerText; const i=b.search(/No templates|No matching/i); return i<0?'(none)':b.slice(i,i+200).replace(/\n/g,' | ');}));
console.log('search box value on Templates:', await page.evaluate(()=>{const i=[...document.querySelectorAll('input')].find(x=>/Search/.test(x.placeholder||'')); return i? i.placeholder+' = '+JSON.stringify(i.value):'none';}));
await app.shot('QA-088-after-templates-query-empty-state');

// --- template import
await page.evaluate(()=>{const i=[...document.querySelectorAll('input')].find(x=>/Search/.test(x.placeholder||'')); if(i){i.value='';i.dispatchEvent(new Event('input',{bubbles:true}));}});
await page.waitForTimeout(600);
console.log('template buttons:', await page.evaluate(()=>[...document.querySelectorAll('main button')].map(b=>b.innerText.trim()).filter(Boolean).slice(0,14)));
const imp = page.getByRole('button', { name: /Import Template|^Import$/ }).first();
await imp.click();
await page.waitForTimeout(800);
const fi = page.locator('input[type=file]');
console.log('file inputs on page:', await fi.count());
console.log('templates before:', await count());
await fi.last().setInputFiles(DIR + 'template-good.json');
await page.waitForTimeout(2000);
console.log('toast:', await toastText(), '| templates after:', await count(), '| present:', (await page.innerText('body')).includes('QA Imported Template'));
await app.shot('QA-088-after-template-import');
for (const f of ['bad.txt','wrong-shape.json']) {
  if (!(await page.locator('[role="dialog"] input[type=file]').count())) { await imp.click().catch(()=>{}); await page.waitForTimeout(700); }
  await page.locator('input[type=file]').last().setInputFiles(DIR + f);
  await page.waitForTimeout(1800);
  console.log(`template import ${f} -> toast:`, await toastText(), '| templates:', await count());
}
await page.keyboard.press('Escape');
await app.done();
