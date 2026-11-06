// document.title + h1 per route
import { open } from './app.mjs';
const app = await open('');
if (!app.mode.startsWith('tauri')) { console.log('NOT TAURI'); process.exit(1); }
for (const r of ['projects','tasks','coder','ai','cloud','logs','sdk','settings','blocks']) {
  await app.page.goto('http://localhost:1420/' + r); await app.page.waitForTimeout(2500);
  const i = await app.page.evaluate(() => ({ title: document.title, h1: [...document.querySelectorAll('h1')].map(h=>h.innerText.trim()), h2: [...document.querySelectorAll('h2')].slice(0,2).map(h=>h.innerText.trim()) }));
  console.log(r, JSON.stringify(i));
}
await app.page.goto('http://localhost:1420/');
await app.done();
