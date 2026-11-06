import { open } from './app.mjs';
const route = process.argv[2], name = process.argv[3], wait = +(process.argv[4]||6000);
const app = await open(route);
await app.page.waitForTimeout(wait);
console.log('URL', app.page.url());
console.log((await app.page.locator('body').innerText()).slice(0,1500));
console.log('shot', await app.shot(name));
await app.done();
