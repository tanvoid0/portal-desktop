// Repro: reload Settings > AI > Providers; count loads that fail to parse ai-settings.json.
import { open } from './app.mjs';
const app = await open('settings/ai/providers');
const { page } = app;
let bad = 0, shot = false;
for (let i = 1; i <= 15; i++) {
  const before = app.errors.length;
  await page.goto("http://tauri.localhost/ai/providers"); // the route that first showed it
  await page.waitForTimeout(2500);
  const errs = app.errors.slice(before).filter((e) => /AI settings|configuration/i.test(e));
  if (errs.length) {
    bad++;
    console.log(`load ${i}: ${errs[0]}`);
    await page.waitForTimeout(6000);
    const stuck = await page.getByText(/^Loading(…|...)$/).count();
    console.log(`  still "Loading..." 8.5s later: ${stuck > 0}`);
    if (!shot) { await app.shot('QA-192-before-providers-stuck'); shot = true; }
  }
}
console.log(`${bad}/15 loads failed`);
console.log(await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('ai_platform_health')));
await app.done();
