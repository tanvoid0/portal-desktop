import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);
const info = await app.page.evaluate(() => {
  const cards = Array.from(document.querySelectorAll('[data-slot="card"]'));
  const matches = cards.filter((c) => c.textContent?.includes('Unnamed device'));
  return matches.map((c) => ({
    buttons: Array.from(c.querySelectorAll('button')).map((b) => JSON.stringify(b.textContent?.trim())),
  }));
});
console.log(JSON.stringify(info, null, 2));
await app.done();
