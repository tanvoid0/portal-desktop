import { open } from './app.mjs';
const app = await open('projects/4/pipelines'); const p = app.page;
console.log(await p.locator('a').evaluateAll(a => a.map(x => x.getAttribute('href')+':'+x.innerText.trim().slice(0,20)).filter(h => /ai|agent|coder/i.test(h))));
await app.done();
