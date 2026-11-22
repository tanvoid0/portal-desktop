import { open } from './app.mjs';
const app = await open('tasks');
const p = app.page;
const dlg = p.locator('[role=dialog]');
try { await dlg.waitFor({ timeout: 4000 }); } catch {}
console.log('dialog text:', (await dlg.allInnerTexts()).join('|').replace(/\n/g,' / '));
console.log(await app.shot('QA-160-after-vault-dialog-reason'));
await p.getByRole('button', { name: 'Not now' }).click(); await p.waitForTimeout(600);
console.log('closed:', await dlg.count()===0);
// client-side nav through non-vault screens
for (const r of ['Documents','Tasks','Action Runs','Terminal']) {
  await p.getByRole('link', { name: r, exact: true }).first().click().catch(()=>{}); await p.waitForTimeout(1800);
  console.log(r, 'dialog re-opened:', await dlg.count());
}
await p.getByRole('link', { name: 'GitHub', exact: true }).first().click().catch(()=>{}); await p.waitForTimeout(2500);
console.log('github url', p.url(), 'dialog:', (await dlg.allInnerTexts()).join('|').replace(/\n/g,' / ').slice(0,200));
console.log(await app.shot('QA-190-after-github-prompt'));
if (await dlg.count()) await p.getByRole('button', { name: 'Not now' }).click();
await p.waitForTimeout(500);
console.log('after cancel dialog:', await dlg.count());
await app.done();
