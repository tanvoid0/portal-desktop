import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);

let revoked = 0;
for (let i = 0; i < 5; i++) {
  const unnamed = app.page.locator('text=Unnamed device').first();
  if ((await unnamed.count()) === 0) break;
  const card = unnamed.locator('xpath=ancestor::*[self::div][3]');
  const revokeBtn = card.locator('button').filter({ hasText: 'Revoke' });
  if ((await revokeBtn.count()) === 0) break;
  await revokeBtn.first().click();
  await app.page.waitForTimeout(500);
  // Dump every visible button so we target the real confirm label, whatever it is.
  const allBtns = await app.page.locator('button:visible').allInnerTexts();
  console.log('visible buttons after clicking Revoke:', JSON.stringify(allBtns));
  // The confirm dialog's destructive action is the one NOT already visible
  // on the page before (i.e. not "Revoke" itself, not nav/sidebar labels).
  const candidate = app.page.locator('[role="alertdialog"] button, [role="dialog"] button').last();
  if ((await candidate.count()) > 0) {
    const label = await candidate.innerText();
    console.log('clicking dialog button labeled:', JSON.stringify(label));
    await candidate.click();
    await app.page.waitForTimeout(700);
    revoked++;
  } else {
    console.log('no dialog found, aborting loop');
    break;
  }
}
console.log('revoked count:', revoked);
const text = await app.page.evaluate(() => document.body.innerText);
console.log('paired count now:', text.match(/Paired\s+(\d+)/)?.[1]);
console.log('still mentions Unnamed device:', text.includes('Unnamed device'));
await app.shot('QA-110-cleanup4-after');
await app.done();
