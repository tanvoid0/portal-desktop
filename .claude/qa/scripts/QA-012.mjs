// QA-012: Settings breadcrumbs show raw URL segments ("Ides", "Github", "Ai", "Frameworks languages").
import { open } from './app.mjs';
const app = await open('settings/ides');
const p = app.page;
for (const r of ['general','ides','github','ai','ai/training','frameworks-languages','package-managers','autonomy']) {
  await p.goto(new URL('/settings/' + r, p.url()).href); await p.waitForTimeout(900);
  console.log(r.padEnd(22), '|', (await p.locator('header').first().innerText()).replace(/\s+/g, ' ').trim());
  if (r === 'ai/training') await app.shot('QA-012-before-settings-crumb-ai-training');
}
await app.done();
