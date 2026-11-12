import { open } from './app.mjs';
const [route, name, ms='4000'] = process.argv.slice(2);
const app = await open(route); await app.page.waitForTimeout(Number(ms));
console.log((await app.page.locator('body').innerText()).slice(0,1500)); console.log(await app.shot(name)); await app.done();
