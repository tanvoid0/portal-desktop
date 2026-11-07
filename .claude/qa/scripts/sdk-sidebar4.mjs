import { open } from './i2.mjs';
const app = await open('sdk/python', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(7000);
console.log('expanded: categories open ->', await page.evaluate(()=>{
  const secs=[...document.querySelectorAll('button')].filter(b=>/^(SDK Managers|Language & Runtime|Database Server|Web Server|Container Platform|AI SDKs)$/.test(b.innerText.trim()));
  return secs.map(b=>b.innerText.trim()+':'+(b.getAttribute('aria-expanded') ?? b.closest('[data-state]')?.getAttribute('data-state') ?? '?'));
}));
console.log('Python row visible:', await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Python')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); return b?Math.round(b.getBoundingClientRect().y):'none';}));
// collapse a category
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Language & Runtime'); b.click();});
await page.waitForTimeout(800);
console.log('after collapsing Language & Runtime, Python row:', await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Python')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); return b?Math.round(b.getBoundingClientRect().y):'none';}));
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Language & Runtime'); b.click();});
await page.waitForTimeout(800);
// click an SDK and a manager
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Go')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); b.click();});
await page.waitForTimeout(2500);
console.log('clicked Go in the tree ->', page.url().replace('http://localhost:1421',''));
// mobile sheet
await app.size(700, 800);
await page.waitForTimeout(1200);
await app.shot('sdk-sidebar-mobile-700');
console.log('at 700px wide, SDK tree visible:', await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim()==='Language & Runtime'); return b? Math.round(b.getBoundingClientRect().width):'hidden';}));
console.log('sheet trigger buttons:', await page.evaluate(()=>[...document.querySelectorAll('button')].filter(b=>/toggle|menu|sdk/i.test((b.getAttribute('aria-label')||'')+b.innerText)).map(b=>b.getAttribute('aria-label')||b.innerText.trim().slice(0,30)).slice(0,8)));
await app.size(1280, 800);
await app.done();
