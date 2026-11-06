// Controls with no accessible name: logs filters + tasks kanban status toggles + script detail back/interactive switch
import { open } from './app.mjs';
const app = await open('logs'); const p = app.page;
const unnamed = (sel) => p.locator(sel).evaluateAll(els => els.filter(x => x.offsetParent && !(x.getAttribute('aria-label') || x.getAttribute('aria-labelledby') || x.title || x.innerText.trim() && x.tagName !== 'BUTTON' || (x.tagName==='BUTTON' && x.getAttribute('role')!=='combobox' && x.innerText.trim()) )).map(x => (x.getAttribute('role') || x.tagName) + ' text=' + JSON.stringify(x.innerText.trim())));
await p.waitForTimeout(2000);
console.log('/logs:', await unnamed('main button, main [role=combobox], main [role=switch]'));
await p.goto('http://localhost:1420/tasks'); await p.waitForTimeout(2000);
const t = await unnamed('main button'); console.log('/tasks unnamed buttons:', t.length);
await p.goto('http://localhost:1420/automation/scripts'); await p.waitForTimeout(1500);
await p.getByRole('button', { name: 'View', exact: true }).first().click(); await p.waitForTimeout(1500);
console.log('script detail:', await unnamed('main button, main [role=switch]'));
await app.shot('QA-069-before-unnamed-controls-script-detail');
await p.goto('http://localhost:1420/logs'); await p.waitForTimeout(1500);
await app.shot('QA-069-before-unnamed-controls-logs');
await app.done();
