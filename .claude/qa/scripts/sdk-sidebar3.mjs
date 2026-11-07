import { open } from './i2.mjs';
const app = await open('sdk/managers', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(8000);
const toggle = async () => { await page.evaluate(()=>{ const bs=[...document.querySelectorAll('button')].filter(b=>/toggle sidebar/i.test(b.innerText+b.getAttribute('aria-label'))); bs[bs.length-1].click(); }); await page.waitForTimeout(1000); };

// tooltip on a rail icon
await toggle();
const railBtn = page.locator('button').filter({ has: page.locator('svg') }).nth(0);
await page.evaluate(()=>{ const b=[...document.querySelectorAll('button')].filter(x=>{const r=x.getBoundingClientRect(); return r.x>255 && r.x<340;})[1]; b.scrollIntoView(); });
const bx = await page.evaluate(()=>{ const b=[...document.querySelectorAll('button')].filter(x=>{const r=x.getBoundingClientRect(); return r.x>255 && r.x<340;})[1]; const r=b.getBoundingClientRect(); return [Math.round(r.x+r.width/2), Math.round(r.y+r.height/2)]; });
await page.mouse.move(bx[0]-30, bx[1]); await page.waitForTimeout(200);
await page.mouse.move(bx[0], bx[1]); await page.waitForTimeout(1500);
console.log('rail tooltip:', JSON.stringify(await page.evaluate(()=>[...document.querySelectorAll('[role="tooltip"],[data-slot="tooltip-content"]')].map(e=>e.innerText.trim()))));
await app.shot('sdk-sidebar-rail-hover');
await toggle();

// active highlight across routes
for (const r of ['sdk','sdk/managers','sdk/managers/nvm','sdk/services','sdk/versions','sdk/python']) {
  await app.go(r); await page.waitForTimeout(3000);
  const act = await page.evaluate(() => [...document.querySelectorAll('[data-active="true"], [aria-current]')].map(e=>e.innerText.trim().split('\n')[0]).filter(Boolean));
  console.log(`/${r} active:`, JSON.stringify(act));
}
// category expand + selected sdk's category open
await app.go('sdk/python'); await page.waitForTimeout(4000);
console.log('on /sdk/python, open categories:', await page.evaluate(()=>[...document.querySelectorAll('[data-state="open"]')].map(e=>e.innerText.trim().split('\n')[0]).filter(Boolean).slice(0,6)));
console.log('python visible in the tree:', await page.evaluate(()=>{ const b=[...document.querySelectorAll('aside button, [data-slot="sidebar"] button')].find(x=>/^Python/.test(x.innerText.trim())); return b? Math.round(b.getBoundingClientRect().width)>0 : false; }));
await app.shot('sdk-sidebar-python-selected');
await toggle();
await page.waitForTimeout(800);
await app.shot('sdk-sidebar-python-selected-rail');
await toggle();
await app.done();
