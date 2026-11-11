import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);
const info = await app.page.evaluate(() => {
  const all = Array.from(document.querySelectorAll('*')).filter(
    (el) => el.children.length === 0 === false && el.textContent?.includes('Unnamed device') === false
  );
  // Find the smallest element containing exactly "Unnamed device" as its own text
  const nameEl = Array.from(document.querySelectorAll('*')).find(
    (el) => el.children.length === 0 && el.textContent?.trim() === 'Unnamed device'
  );
  const path = [];
  let cur = nameEl;
  for (let i = 0; i < 6 && cur; i++) {
    path.push({ tag: cur.tagName, cls: cur.className?.toString().slice(0, 60), dataSlot: cur.getAttribute?.('data-slot') });
    cur = cur.parentElement;
  }
  return path;
});
console.log(JSON.stringify(info, null, 2));
await app.done();
