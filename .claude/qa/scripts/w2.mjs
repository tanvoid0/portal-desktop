import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('github');
const p = app.page;
await dismissVault(p, 4000); await p.waitForTimeout(2000);
await p.evaluate(()=>document.activeElement?.blur());
let hit=null;
for (let i=0;i<60;i++){ await p.keyboard.press('Tab'); const t=await p.evaluate(()=>{const e=document.activeElement; const cs=getComputedStyle(e); return {tag:e.tagName, href:e.getAttribute('href'), ring:(cs.outlineStyle!=='none'&&parseFloat(cs.outlineWidth)>0)||cs.boxShadow!=='none'}}); if(t.href==='/github/repos'){hit=t;break;} }
console.log('165 focus on repos card:', hit);
console.log(await app.shot('QA-165-after2-github-focus-ring'));
if (hit) { await p.keyboard.press('Enter'); await p.waitForTimeout(1500); console.log('Enter ->', p.url()); }
// 169/174/180/176
const go = async (r) => { await p.goto(new URL('/'+r, p.url()).href); await dismissVault(p,1500); await p.waitForTimeout(1800); };
await go('automation/utilities');
console.log('174 tab labels:', await p.evaluate(()=>[...document.querySelectorAll('[role=tab],a')].map(e=>e.textContent.trim()).filter(t=>/Utilit|Scripts/.test(t))));
await go('utilities/disk');
console.log(await app.shot('QA-180-after2-disk-1280'));
await p.setViewportSize({width:1024,height:600}); await p.waitForTimeout(800);
console.log(await app.shot('QA-180-after2-disk-1024'));
await p.setViewportSize({width:1280,height:800});
await go('logs');
console.log('176 header:', await p.evaluate(()=>document.querySelector('h1')?.textContent.trim()));
await p.getByRole('button', { name: 'Clear logs' }).click(); await p.waitForTimeout(500);
await p.locator('[role=alertdialog],[role=dialog]').getByRole('button', { name: /^Clear/ }).click(); await p.waitForTimeout(700);
console.log('176 toast:', await p.locator('[data-sonner-toast]').allInnerTexts(), 'count:', await p.evaluate(()=>document.body.innerText.match(/Showing \d+ of \d+/)?.[0]));
console.log(await app.shot('QA-176-after2-logs-cleared'));
await app.done();
