import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('projects/233');
const p = app.page;
await dismissVault(p);
console.log(await app.shot('sweep-projects-detail-233'));
console.log('tabs/buttons', await p.locator('main button, main [role=tab]').evaluateAll(els=>els.map(e=>(e.getAttribute('aria-label')||e.title||e.textContent).trim().replace(/\s+/g,' ').slice(0,30)).filter(Boolean).slice(0,40)));
await app.done();
