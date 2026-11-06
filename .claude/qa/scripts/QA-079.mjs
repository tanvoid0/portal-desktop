import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
for (const [w,h] of [[1280,800],[1024,600]]) {
  await p.setViewportSize({width:w,height:h}); await p.waitForTimeout(400);
  await p.keyboard.press('Control+f'); await p.waitForTimeout(500);
  const m = await p.evaluate(()=>{const d=document.querySelector('[role=dialog]');const r=d.getBoundingClientRect();const out={dlg:[Math.round(r.top),Math.round(r.bottom)]};for(const n of ['Clear All','Cancel','Apply']){const b=[...d.querySelectorAll('button')].find(b=>b.textContent.includes(n));const br=b.getBoundingClientRect();out[n]=[Math.round(br.top),Math.round(br.bottom),Math.round(br.left),Math.round(br.right)]}
   out.scrollers=[...d.querySelectorAll('*')].filter(e=>e.scrollHeight>e.clientHeight+2&&/auto|scroll/.test(getComputedStyle(e).overflowY)).map(e=>e.scrollHeight+'/'+e.clientHeight);return out});
  console.log(w,h,JSON.stringify(m)); await app.shot(`QA-079-after-filter-dialog-${w}x${h}`);
  await p.keyboard.press('Escape'); await p.waitForTimeout(300);
}
// apply a filter at 1024x600 by mouse
await p.keyboard.press('Control+f'); await p.waitForTimeout(500);
const d=p.getByRole('dialog');
const sc=await p.evaluate(()=>{const d=document.querySelector('[role=dialog]');const e=[...d.querySelectorAll('*')].find(e=>e.scrollHeight>e.clientHeight+2&&/auto|scroll/.test(getComputedStyle(e).overflowY));if(!e)return null;e.scrollTop=e.scrollHeight;return e.scrollTop});
console.log('scrolled', sc); await p.waitForTimeout(300);
await app.shot('QA-079-after-filter-dialog-scrolled-1024x600');
await d.getByText('Bug',{exact:true}).first().click().catch(e=>console.log('bug click fail'));
await p.getByRole('button',{name:/Apply/}).click(); await p.waitForTimeout(800);
console.log('dialogs', await p.getByRole('dialog').count());
const cards=await p.locator('[aria-label^="Select task: QA"]').evaluateAll(a=>a.map(x=>x.getAttribute('aria-label')));
console.log('QA cards after Bug filter', cards, 'chips', (await p.locator('main').innerText()).match(/Type[^\n]*|Bug[^\n]*/g));
await app.shot('QA-079-after-applied-bug-filter');
await app.done();
