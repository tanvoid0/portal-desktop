import { open } from './i2.mjs';
const app = await open('sdk', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(9000);
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Language & Runtime'); b.click();});
await page.waitForTimeout(900);
console.log('after clicking Language & Runtime:', await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Language & Runtime'); const py=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Python')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); return b.getAttribute('aria-expanded')+' | python row: '+(py?'visible':'no');}));
await app.shot('sdk-sidebar-category-expanded');
await page.evaluate(()=>{const py=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Python')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); py.click();});
await page.waitForTimeout(3000);
console.log('clicked Python ->', page.url().replace('http://localhost:1421',''));
console.log('category still open after nav:', await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Language & Runtime'); return b.getAttribute('aria-expanded');}));
// manager
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='SDK Managers'&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); b.click();});
await page.waitForTimeout(900);
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>/^NVM/.test(x.innerText.trim())&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); if(b) b.click();});
await page.waitForTimeout(3000);
console.log('clicked NVM in the tree ->', page.url().replace('http://localhost:1421',''));
await app.done();
