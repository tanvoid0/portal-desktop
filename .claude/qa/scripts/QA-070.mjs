// Quick Run: Preview does nothing visible (bad folder and real folder)
import { open } from './app.mjs';
const app = await open('automation/run'); const p = app.page;
await p.waitForTimeout(1500);
const dir = p.getByPlaceholder('D:/projects/my-app');
await p.getByRole('button', { name: 'Test', exact: true }).click();
for (const d of [String.raw`C:\no\such\qa-dir`, String.raw`C:\Users\tanvo\AppData\Local\Temp`]) {
  await dir.fill(d);
  const before = await p.locator('main').innerText();
  await p.getByRole('button', { name: 'Preview' }).click();
  await p.waitForTimeout(3000);
  const after = await p.locator('main').innerText();
  const toasts = (await p.locator('[aria-live=polite] [role=alert]').allInnerTexts()).join('|');
  console.log(d, '-> page changed:', before !== after, 'toast:', JSON.stringify(toasts), 'dialog:', await p.getByRole('dialog').count());
}
await app.done();
