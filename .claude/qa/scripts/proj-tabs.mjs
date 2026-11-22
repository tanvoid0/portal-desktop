import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('projects/233');
const p = app.page;
await dismissVault(p);
await p.getByRole('button', { name: 'More' }).click();
await p.waitForTimeout(500);
console.log('menu', await p.locator('[role=menuitem]').allInnerTexts());
console.log(await app.shot('sweep-projects-more-menu'));
await p.keyboard.press('Escape');
for (const t of ['Actions', 'Env', 'Terminal']) {
  await p.getByRole('tab', { name: t }).click().catch(()=>p.getByRole('button',{name:t}).first().click());
  await p.waitForTimeout(1200);
  console.log(t, p.url());
  console.log(await app.shot('sweep-projects-233-tab-' + t.toLowerCase()));
}
await app.done();
