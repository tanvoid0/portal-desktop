import { open } from './i2.mjs';
const DIR = 'C:/Users/tanvo/AppData/Local/Temp/claude/660cd8302a863ca1/2609a446-530c-4f61-83da-a545cae02459/scratchpad/qafiles/';
const app = await open('automation/blocks', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(2500);
const toastText = () => page.evaluate(() => (document.querySelector('[aria-label="Notifications"]')?.innerText||'').replace(/\n/g,' | ').trim());
const count = () => page.evaluate(() => [...document.querySelectorAll('[data-slot="card"]')].filter(c=>c.getBoundingClientRect().y>150).length);

// blocks empty state wording
await page.getByPlaceholder('Search blocks...').fill('zzzznope');
await page.waitForTimeout(700);
console.log('blocks empty state:', await page.evaluate(()=>{const c=[...document.querySelectorAll('[data-slot="card"]')].filter(x=>x.getBoundingClientRect().y>150)[0]; return c?c.innerText.replace(/\n/g,' | ').slice(0,220):'(no card)';}));
await page.getByPlaceholder('Search blocks...').fill('');
await page.waitForTimeout(500);

// templates: valid import
await page.evaluate(()=>{const b=[...document.querySelectorAll('button,[role="tab"]')].find(x=>x.innerText.trim()==='Templates'); b.click();});
await page.waitForTimeout(1200);
console.log('templates before:', await count());
await page.getByRole('button', { name: /Import Template/ }).first().click();
await page.waitForTimeout(700);
await page.locator('input[type=file]').last().setInputFiles(DIR + 'template-valid.json');
await page.waitForTimeout(2000);
console.log('valid template import toast:', await toastText(), '| after:', await count(), '| present:', (await page.innerText('body')).includes('QA Imported Template'));
await app.shot('QA-088-after-valid-template-import');
// template export + edit + delete of the custom one
const card = page.locator('[data-slot="card"]').filter({ hasText: 'QA Imported Template' }).first();
console.log('custom template card buttons:', await card.evaluate(c=>[...c.querySelectorAll('button')].map(b=>b.getAttribute('aria-label')||b.innerText.trim()||'(icon)')));
const dl = page.waitForEvent('download', {timeout:5000}).catch(()=>null);
await card.locator('button').nth(1).click();
await page.waitForTimeout(1200);
const d = await dl;
console.log('export:', d? d.suggestedFilename() : 'no download; toast=' + await toastText());
await page.keyboard.press('Escape');
await app.done();
