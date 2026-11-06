// QA-019: /logs shows "No logs found" / "Showing 0 of 9 logs" with both filters on "all".
import { open } from './app.mjs';
const app = await open('logs');
const p = app.page;
await p.waitForTimeout(1500);
const txt = await p.locator('main').first().innerText();
console.log('footer:', txt.match(/Showing \d+ of \d+ logs/)?.[0], '| empty message:', /No logs found/.test(txt));
console.log('filters:', await p.getByRole('combobox').allInnerTexts());
await app.shot('QA-019-before-logs-empty-but-9');
await app.done();
