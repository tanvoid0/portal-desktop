import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open(process.argv[2]);
await dismissVault(app.page);
console.log(await app.page.evaluate(()=>[...document.body.innerText.matchAll(/QA [^\n]{0,40}/g)].map(m=>m[0])));
await app.done();
