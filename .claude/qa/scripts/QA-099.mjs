import { open } from './app.mjs';

const app = await open('sdk/python');
await app.page.waitForTimeout(1500);
const geom = await app.page.evaluate(() => {
  const el = [...document.querySelectorAll('*')].find(
    (e) => e.textContent.trim() === 'Development Tools' && e.children.length === 0
  );
  if (!el) return null;
  const rect = el.getBoundingClientRect();
  let clipper = el.parentElement;
  while (clipper && getComputedStyle(clipper).overflow !== 'hidden') clipper = clipper.parentElement;
  const crect = clipper ? clipper.getBoundingClientRect() : null;
  return {
    textTop: rect.top,
    textBottom: rect.bottom,
    clipperBottom: crect ? crect.bottom : null,
    clipperClass: clipper ? clipper.className : null,
  };
});
console.log('subtitle geometry:', JSON.stringify(geom));
await app.shot('QA-099-sidebar-header');
await app.done();
