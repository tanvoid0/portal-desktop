import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('tasks');
const p = app.page;
await dismissVault(p);
const info = await p.evaluate(() => { const t = [...document.querySelectorAll('h3,h4,p,span')].find(e=>/release_rehearse refuses/.test(e.textContent)&&e.children.length===0); let e=t; const chain=[]; while(e&&e!==document.body){ chain.push(e.tagName+'['+(e.getAttribute('role')||'')+','+(e.getAttribute('tabindex')||'')+']'); e=e.parentElement; if(chain.length>8)break;} return chain; });
console.log(info.join(' < '));
// tab count to reach first card
let n=0, found=null;
for (let i=0;i<120;i++){ await p.keyboard.press('Tab'); const t = await p.evaluate(()=>document.activeElement?.textContent?.trim().slice(0,40)); if(/release_rehearse/.test(t)){found=i+1;break;} }
console.log('tabs to first card', found);
if (found) { await p.keyboard.press('Enter'); await p.waitForTimeout(800); console.log('Enter ->', p.url()); await p.keyboard.press('Escape'); await p.waitForTimeout(500); console.log('Esc ->', p.url()); }
await app.done();
