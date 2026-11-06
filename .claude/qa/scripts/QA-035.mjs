// Repro: after saving from a long form, the next page opens scrolled down (header cut off).
import { open } from './app.mjs';
const app = await open('tasks/36/edit');
const p = app.page;
await p.waitForTimeout(800);
const pane = () => p.evaluate(() => { const s = [...document.querySelectorAll('*')].filter((e) => e.scrollHeight > e.clientHeight + 20 && /auto|scroll/.test(getComputedStyle(e).overflowY)); return s.map((e) => e.tagName + ':' + e.scrollTop); });
await p.evaluate(() => [...document.querySelectorAll('*')].forEach((e) => { if (e.scrollHeight > e.clientHeight + 20 && /auto|scroll/.test(getComputedStyle(e).overflowY)) e.scrollTop = 9999; }));
console.log('edit page scrollTops:', await pane());
await p.getByRole('button', { name: /Update Task|Save/ }).last().click();
await p.waitForTimeout(1500);
console.log(p.url(), 'detail scrollTops:', await pane(), '(expected 0)');
await app.shot('QA-035-before-detail-opens-scrolled');
await app.done();
