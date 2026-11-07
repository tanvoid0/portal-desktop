import { open } from './i2.mjs';
const DIR = 'C:/Users/tanvo/AppData/Local/Temp/claude/660cd8302a863ca1/2609a446-530c-4f61-83da-a545cae02459/scratchpad/qafiles/';
const app = await open('scripts/new', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(1500);
console.log('header buttons on /scripts/new:', await page.evaluate(()=>[...document.querySelectorAll('main button')].map(b=>b.innerText.trim()).filter(Boolean).slice(0,8)));
await page.getByRole('button', { name: /Import JSON/i }).first().click();
await page.waitForTimeout(700);
console.log('dialog:', await page.evaluate(()=>document.querySelector('[role="dialog"]')?.innerText.replace(/\n/g,' | ').slice(0,200)));
const ta = page.locator('[role="dialog"] textarea').first();
console.log('paste textarea present:', await ta.count(), '| file input present:', await page.locator('[role="dialog"] input[type=file]').count());
await ta.fill(JSON.stringify({name:'QA Pasted Script',description:'QA',script_type:'shell',content:'echo QA',parameters:[]}));
const impBtns = await page.evaluate(()=>[...document.querySelectorAll('[role="dialog"] button')].map(b=>b.innerText.trim()).filter(Boolean));
console.log('dialog buttons:', JSON.stringify(impBtns));
await page.getByRole('dialog').getByRole('button', { name: 'Import & Preview' }).click();
await page.waitForTimeout(1200);
console.log('name field after paste import:', await page.evaluate(()=>{const i=[...document.querySelectorAll('input')].find(x=>/name/i.test(x.id+x.name+(x.placeholder||''))); return i?i.value:'?';}));
await app.shot('UX-016-after-scripts-new-paste-import');
// file import
await page.getByRole('button', { name: /Import JSON/i }).first().click();
await page.waitForTimeout(700);
await page.locator('[role="dialog"] input[type=file]').setInputFiles(DIR + 'script-good.json');
await page.waitForTimeout(1500);
console.log('name field after file import:', await page.evaluate(()=>{const i=[...document.querySelectorAll('input')].find(x=>/name/i.test(x.id+x.name+(x.placeholder||''))); return i?i.value:'?';}));
// ?import=true
await app.go('scripts/new?import=true');
await page.waitForTimeout(1500);
console.log('?import=true auto-opens dialog:', await page.locator('[role="dialog"]').count());
await page.keyboard.press('Escape');
// deployments
await app.go('deployments/new?projectPath=D:/projects/qa-demo');
await page.waitForTimeout(1500);
console.log('deployments header buttons:', await page.evaluate(()=>[...document.querySelectorAll('main button')].map(b=>b.innerText.trim()).filter(Boolean).slice(0,8)));
console.log('projectPath prefilled:', await page.evaluate(()=>[...document.querySelectorAll('input')].map(i=>i.value).filter(v=>v && v.includes('qa-demo'))));
await app.shot('UX-016-after-deployments-new');
await app.done();
