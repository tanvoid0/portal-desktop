// /settings/ides: edit, delete (with mappings), mapping quick-add/edit/delete, custom framework.
import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;
const BS = String.fromCharCode(92);

// collect toasts continuously
const seen = [];
await page.exposeFunction('qaToast', (t) => seen.push(t)).catch(() => {});
async function watchToasts() {
  await page.evaluate(() => {
    if (window.__qaObs) return;
    const root = document.querySelector('[aria-label="Notifications"]');
    if (!root) return;
    window.__qaObs = new MutationObserver(() => {
      for (const n of root.children) {
        const t = n.innerText.replace(/\n/g, ' | ').trim();
        if (t) window.qaToast(t);
      }
    });
    window.__qaObs.observe(root, { childList: true, subtree: true });
  });
}
const drain = () => { const s = [...new Set(seen)]; seen.length = 0; return s; };
await watchToasts();

const rows = () => page.locator('div.rounded-md.border.p-4');
const rowTexts = () => page.evaluate(() => [...document.querySelectorAll('div.rounded-md.border.p-4')].map(d => d.innerText.replace(/\n/g,' | ')));

async function rowByName(name) {
  const n = await rows().count();
  for (let i = 0; i < n; i++) if ((await rows().nth(i).innerText()).includes(name)) return rows().nth(i);
  return null;
}

console.log('=== EDIT an IDE ===');
let r = await rowByName('QA Path Test');
await r.locator('button').nth(1).click();     // pencil
await page.waitForTimeout(600);
await page.locator('#ide-name').fill('QA Path Test renamed');
await page.locator('#ide-executable').fill('C:' + BS + 'Tools' + BS + 'QA' + BS + 'renamed.exe');
console.log('field now:', JSON.stringify(await page.locator('#ide-executable').inputValue()));
await page.getByRole('dialog').getByRole('button', { name: 'Update IDE' }).click();
await page.waitForTimeout(1500);
console.log('toasts:', JSON.stringify(drain()));
console.log('rows:', JSON.stringify(await rowTexts()));
await app.go('settings/ides');
await watchToasts();
console.log('rows after reload:', JSON.stringify(await rowTexts()));

console.log('=== quick-add a mapping from a framework chip ===');
await page.getByRole('button', { name: 'React', exact: true }).first().click();
await page.waitForTimeout(1500);
console.log('toasts:', JSON.stringify(drain()));
await app.shot('ides-after-quickadd-mapping');
const mapText = async () => page.evaluate(() => { const b=document.body.innerText; const i=b.lastIndexOf('Framework IDE Mappings'); return b.slice(i, i+300).replace(/\n/g,' | '); });
console.log('mappings:', await mapText());

console.log('=== edit the mapping ===');
const mapRow = await page.evaluate(() => {
  const els = [...document.querySelectorAll('div')].filter(d => /react/i.test(d.innerText) && d.querySelectorAll('button').length >= 2 && d.innerText.length < 200);
  return els.length;
});
console.log('candidate mapping rows:', mapRow);
await app.shot('ides-mappings-after-add');
await app.done();
