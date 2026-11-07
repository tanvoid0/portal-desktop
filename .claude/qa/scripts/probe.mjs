import { open } from './i2.mjs';
const route = process.argv[2] ?? 'logs';
const app = await open(route);
const links = await app.page.evaluate(() => [...document.querySelectorAll('a[href]')].map(a => a.getAttribute('href') + ' | ' + a.textContent.trim().slice(0,40)));
console.log(links.join('\n'));
await app.done();
