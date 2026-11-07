import { open } from './i2.mjs';
const app = await open('sdk/managers', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(8000);
const toggles = await page.evaluate(()=>[...document.querySelectorAll('button')].filter(b=>/toggle sidebar/i.test(b.innerText+b.getAttribute('aria-label'))).map((b,i)=>i+':'+Math.round(b.getBoundingClientRect().x)));
console.log('toggle sidebar buttons at x:', JSON.stringify(toggles));
// the SDK one is the right-hand toggle
await page.evaluate(()=>{ const bs=[...document.querySelectorAll('button')].filter(b=>/toggle sidebar/i.test(b.innerText+b.getAttribute('aria-label'))); bs[bs.length-1].click(); });
await page.waitForTimeout(1200);
await app.shot('sdk-sidebar-collapsed-rail');
const rail = await page.evaluate(() => {
  const side = [...document.querySelectorAll('[data-slot="sidebar"],[data-sidebar="sidebar"],aside')].map(e=>({cls:e.className.toString().slice(0,40), w:Math.round(e.getBoundingClientRect().width), x:Math.round(e.getBoundingClientRect().x)}));
  return side;
});
console.log('sidebars:', JSON.stringify(rail));
// hover a rail icon for a tooltip
const icons = await page.evaluate(()=>{ const bs=[...document.querySelectorAll('button')].filter(b=>b.getBoundingClientRect().x>255 && b.getBoundingClientRect().x<340); return bs.length; });
console.log('rail buttons in the 255-340 band:', icons);
await page.mouse.move(300, 200); await page.waitForTimeout(1200);
console.log('tooltip:', await page.evaluate(()=>[...document.querySelectorAll('[role="tooltip"],[data-slot="tooltip-content"]')].map(e=>e.innerText.trim())));
await app.shot('sdk-sidebar-rail-tooltip');
// expand again
await page.evaluate(()=>{ const bs=[...document.querySelectorAll('button')].filter(b=>/toggle sidebar/i.test(b.innerText+b.getAttribute('aria-label'))); bs[bs.length-1].click(); });
await page.waitForTimeout(1200);
console.log('expanded again, footer:', await page.evaluate(()=>(document.body.innerText.match(/\d+ of \d+ SDKs/)||['none'])[0]));
await app.done();
