// UX-006 part 3: level/source/context selects, Export download, Clear.
import { open } from './i2.mjs';
const app = await open('logs', { width: 1280, height: 800 });
const page = app.page;
const showing = () => page.evaluate(() => (document.body.innerText.match(/Showing \d+ of \d+/)||[null])[0]);

async function pick(label, option) {
  await page.getByRole('combobox', { name: label }).click();
  await page.waitForTimeout(300);
  await page.getByRole('option', { name: option, exact: true }).first().click();
  await page.waitForTimeout(400);
  console.log(`${label} -> ${option}:`, await showing(), '| trigger text:',
    (await page.getByRole('combobox', { name: label }).innerText()).trim());
}
console.log('baseline:', await showing());
await pick('Filter by level', 'WARN');
await pick('Filter by level', 'All levels');
await pick('Filter by source', 'backend');
await pick('Filter by source', 'All sources');
const ctx = page.getByRole('combobox', { name: 'Filter by context' });
if (await ctx.count()) { await pick('Filter by context', 'AppLayout'); await pick('Filter by context', 'All contexts'); }

const dl = page.waitForEvent('download', { timeout: 5000 }).catch(() => null);
await page.getByRole('button', { name: 'Export logs' }).click();
const d = await dl;
console.log('export download:', d ? d.suggestedFilename() : 'NO DOWNLOAD EVENT');

await page.getByRole('button', { name: 'Clear logs' }).click();
await page.waitForTimeout(500);
console.log('after Clear:', await showing(), '| body says:', (await page.innerText('body')).includes('No logs found'));
await app.shot('UX-006-after-logs-cleared');
await app.done();
