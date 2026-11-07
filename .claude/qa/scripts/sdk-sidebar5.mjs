import { open } from './i2.mjs';
const app = await open('sdk/python', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(12000);
const st = async (tag) => console.log(tag, await page.evaluate(()=>{
  const secs=[...document.querySelectorAll('button')].filter(b=>/^(SDK Managers|Language & Runtime|Database Server|Web Server|Container Platform|AI SDKs)$/.test(b.innerText.trim()));
  const py=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Python')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580);
  return JSON.stringify({ sections: secs.map(b=>b.innerText.trim()+'='+b.getAttribute('aria-expanded')), pythonRowVisible: !!py, loading: document.body.innerText.includes('Loading SDKs') });
}));
await st('direct load /sdk/python after 12s:');
await app.shot('sdk-sidebar-python-direct-load');
// now go to /sdk (warm) then click Python in the tree
await app.go('sdk'); await page.waitForTimeout(9000);
await st('on /sdk:');
await page.evaluate(()=>{const b=[...document.querySelectorAll('button')].find(x=>x.innerText.trim().startsWith('Python')&&x.getBoundingClientRect().x>255&&x.getBoundingClientRect().x<580); if(b) b.click();});
await page.waitForTimeout(3000);
console.log('clicked Python in the tree ->', page.url().replace('http://localhost:1421',''));
await st('after client-side nav to python:');
await app.shot('sdk-sidebar-python-client-nav');
await app.done();
