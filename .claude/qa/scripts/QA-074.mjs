// /coder sessions sidebar: "N sessions" header disagrees with the rows listed under it
import { open } from './app.mjs';
const app = await open('coder'); const p = app.page;
await p.waitForTimeout(3000);
const t = await p.locator('main').first().innerText();
console.log('header:', t.match(/\d+ sessions?/)?.[0]);
const rows = await p.locator('main').first().getByText(/\d+ messages?/).count();
console.log('session rows with a message count:', rows);
await app.shot('QA-074-before-sessions-count');
await app.done();
