import { open } from './app.mjs';
const app = await open('cloud/bogus');
const t = await app.page.locator('main, body').first().innerText();
console.log(/Not Found/.test(t) ? 'BUG: bare unstyled 404, no back/home action' : 'ok');
console.log('links in main content:', await app.page.locator('main a, main button').count());
await app.done();
