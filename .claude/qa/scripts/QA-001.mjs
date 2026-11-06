// Projects list: filter selects should sit on one row with the search box, not one per row.
import { open } from './app.mjs';
const app = await open('projects');
const ys = await app.page.locator('button[role=combobox]').evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
console.log('combobox tops:', ys.join(','), new Set(ys).size === ys.length ? 'BUG: each on its own row' : 'ok');
await app.shot('QA-001-filters');
await app.done();
