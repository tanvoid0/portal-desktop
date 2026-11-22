import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('projects/edit/234');
const p = app.page;
await dismissVault(p);
await p.waitForTimeout(1500);
console.log(await app.shot('sweep-projects-edit-234'));
console.log(await p.locator('main button').evaluateAll(els=>els.map(e=>(e.getAttribute('aria-label')||e.textContent).trim()).filter(Boolean)));
await app.done();
