// #267: Agents Run cwd = searchable project picker, last-used default, free text.
import { open } from './app.mjs';
const app = await open('agents/run');
const p = app.page;
await p.waitForTimeout(1500);
const KEY = 'portal-agents-last-cwd';
const trig = p.locator('button[role="combobox"]').filter({ hasNotText: /Default|Select an agent|Claude/ });
console.log('combobox triggers:', await p.locator('button[role="combobox"]').allInnerTexts());
console.log('stored last cwd:', await p.evaluate((k) => localStorage.getItem(k), KEY));
const cwdTrigger = p.locator('button[role="combobox"]').filter({ hasText: / — |Select a project/ }).first();
console.log('cwd trigger text:', JSON.stringify(await cwdTrigger.innerText().catch(() => '(none)')));
await cwdTrigger.click();
const input = p.getByPlaceholder('Type to filter projects…');
await input.waitFor({ timeout: 5000 });
const all = await p.locator('[data-slot="command-item"]').count();
await input.fill('portal');
await p.waitForTimeout(400);
const filtered = await p.locator('[data-slot="command-item"]').allInnerTexts();
console.log(`items total=${all}, filtered by "portal"=${filtered.length}`, filtered.slice(0, 3).map((s) => s.replace(/\n/g, ' | ')));
await app.shot('QA-267-filtered');
if (filtered.length) {
  await p.locator('[data-slot="command-item"]').first().click();
  await p.waitForTimeout(500);
  console.log('picked; stored:', await p.evaluate((k) => localStorage.getItem(k), KEY));
}
await p.reload(); await p.waitForLoadState('load'); await p.waitForTimeout(1500);
console.log('after reload default trigger:', JSON.stringify(await p.locator('button[role="combobox"]').filter({ hasText: / — |Select a project/ }).first().innerText().catch(() => '(none)')));
console.log('browse button present:', await p.getByTitle('Browse for a folder').count());
await app.done();
