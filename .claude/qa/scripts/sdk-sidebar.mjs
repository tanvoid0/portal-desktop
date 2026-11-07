import { open } from './i2.mjs';
const app = await open('sdk', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(9000);
await app.shot('sdk-overview-loaded-1280');
// header clipping
const hdr = await page.evaluate(() => {
  const el = [...document.querySelectorAll('*')].find(e => e.children.length === 0 && e.textContent.trim() === 'Manage your development environment');
  if (!el) return 'not found';
  const r = el.getBoundingClientRect();
  let p = el.parentElement, clip = null;
  while (p) { const cs = getComputedStyle(p); if (cs.overflow !== 'visible') { const pr = p.getBoundingClientRect(); clip = { cls: p.className.toString().slice(0,60), bottom: Math.round(pr.bottom), overflow: cs.overflow }; break; } p = p.parentElement; }
  return { text: el.textContent.trim(), top: Math.round(r.top), bottom: Math.round(r.bottom), clipper: clip, visibleFully: clip ? r.bottom <= clip.bottom + 0.5 : true };
});
console.log('sidebar header subtitle:', JSON.stringify(hdr));
// is the header a button?
console.log('header element:', await page.evaluate(() => {
  const el = [...document.querySelectorAll('*')].find(e => e.children.length===0 && e.textContent.trim()==='SDK Manager' && e.closest('aside,nav,div'));
  const b = el?.closest('button,a');
  return b ? b.tagName + ' | ' + (b.getAttribute('aria-label')||b.innerText.trim().replace(/\n/g,' / ')) + ' | href=' + b.getAttribute('href') : 'no button ancestor';
}));
// nav items + footer count
console.log('sdk nav:', await page.evaluate(() => {
  const nav = [...document.querySelectorAll('*')].find(e => e.children.length===0 && e.textContent.trim()==='Navigation');
  const box = nav?.parentElement?.parentElement;
  return box ? [...box.querySelectorAll('button,a')].map(b=>b.tagName+':'+b.innerText.trim().replace(/\n/g,' / ')) : 'none';
}));
console.log('footer count text:', await page.evaluate(() => (document.body.innerText.match(/\d+ of \d+ SDKs?/)||['none'])[0]));
await app.done();
