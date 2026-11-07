// /settings/ides behaviour sweep after the 1187->47 line split.
import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const txt = () => page.innerText('body');
const toasts = async () => page.evaluate(() => [...document.querySelectorAll('[data-sonner-toast], [role="status"], li[data-type]')].map(e=>e.innerText.replace(/\n/g,' | ').slice(0,120)));
const count = async (label) => page.evaluate((l) => {
  const h = [...document.querySelectorAll('div')].find(d => d.innerText?.trim().startsWith(l) && d.querySelector('span'));
  return null;
}, label);

async function ideRows() {
  return page.evaluate(() => [...document.querySelectorAll('div.rounded-md.border.p-4')].map(d => d.innerText.replace(/\n/g,' | ')));
}

console.log('--- empty state ---');
console.log('body has "No IDEs configured":', (await txt()).includes('No IDEs configured'));
console.log('rows:', JSON.stringify(await ideRows()));
console.log('mappings empty text present:', /No framework mappings|No mappings/i.test(await txt()));

console.log('--- add an IDE ---');
await page.getByRole('button', { name: 'Add Manually' }).click();
await page.waitForTimeout(500);
await page.locator('#ide-name').fill('QA Editor');
await page.locator('#ide-executable').fill('C:\Program Files\QA Editor\qa editor.exe');
await page.getByRole('dialog').getByRole('button', { name: 'Add IDE' }).click();
await page.waitForTimeout(1200);
console.log('toasts:', JSON.stringify(await toasts()));
console.log('rows:', JSON.stringify(await ideRows()));
await app.shot('ides-after-add');

console.log('--- persists across reload ---');
await app.go('settings/ides');
console.log('rows after reload:', JSON.stringify(await ideRows()));

console.log('--- icon button accessible names ---');
const names = await page.evaluate(() => [...document.querySelectorAll('div.rounded-md.border.p-4 button')].map(b => ({ text: b.innerText.trim(), label: b.getAttribute('aria-label'), title: b.getAttribute('title') })));
console.log(JSON.stringify(names));

console.log('--- set default ---');
await page.getByRole('button', { name: 'Set Default' }).first().click();
await page.waitForTimeout(1200);
console.log('toasts:', JSON.stringify(await toasts()));
console.log('rows:', JSON.stringify(await ideRows()));

console.log('--- detect IDEs ---');
await page.getByRole('button', { name: 'Detect IDEs' }).first().click();
await page.waitForTimeout(6000);
console.log('toasts:', JSON.stringify(await toasts()));
console.log('detected section:', (await txt()).includes('Detected IDEs'));
const chips = await page.evaluate(() => {
  const h = [...document.querySelectorAll('p')].find(p => p.innerText.trim() === 'Detected IDEs');
  if (!h) return null;
  return [...h.parentElement.querySelectorAll('button')].map(b => b.innerText.trim());
});
console.log('chips:', JSON.stringify(chips));
await app.shot('ides-after-detect');
await app.done();
