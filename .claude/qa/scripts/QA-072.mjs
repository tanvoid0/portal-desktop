// Script import: JSON that is not a script ([1,2,3], {"foo":1}) reports success and leaves the form empty
import { open } from './app.mjs';
const app = await open('scripts/new?import=true'); const p = app.page;
for (const junk of ['[1,2,3]', '{"foo": 1}']) {
  await p.goto('http://localhost:1420/scripts/new?import=true'); await p.waitForTimeout(1500);
  await p.locator('textarea').first().fill(junk);
  await p.getByRole('button', { name: 'Import & Preview' }).click();
  const s = new Set(); for (let i = 0; i < 8; i++) { await p.waitForTimeout(250); (await p.locator('[aria-live=polite] [role=alert]').allInnerTexts()).forEach(t => t.trim() && s.add(t.replace(/\s+/g, ' ').trim())); }
  console.log(junk, '-> toast:', [...s].join('|') || '(none)', '| Name field:', JSON.stringify(await p.locator('input').nth(0).inputValue()));
  if (junk === '[1,2,3]') await app.shot('QA-072-before-import-array-says-imported');
}
await app.done();
