// Does clicking a framework chip open the "add mapping" dialog, or does the invisible
// hover overlay (edit / delete) swallow the click?
import { open } from './i2.mjs';
const app = await open('settings/ides', { width: 1280, height: 800 });
const page = app.page;

const chip = page.getByRole('button', { name: 'React', exact: true }).first();
await chip.scrollIntoViewIfNeeded();
const box = await chip.boundingBox();
const overlay = await page.evaluate(() => {
  const b = [...document.querySelectorAll('button')].find(x => x.innerText.trim() === 'React');
  const ov = b.parentElement.querySelector('div.absolute');
  const r = ov.getBoundingClientRect(); const br = b.getBoundingClientRect();
  return { chip: {x:Math.round(br.x), w:Math.round(br.width)}, overlay: {x:Math.round(r.x), w:Math.round(r.width)},
           opacity: getComputedStyle(ov).opacity, pe: getComputedStyle(ov).pointerEvents };
});
console.log('geometry:', JSON.stringify(overlay));
console.log('chip box:', JSON.stringify(box));
console.log('element at chip centre:', await page.evaluate(([x,y]) => {
  const e = document.elementFromPoint(x,y);
  return e.tagName + ' | ' + (e.closest('button')?.innerText.trim() ?? '(no button)') + ' | ' + (e.closest('div.absolute') ? 'INSIDE HOVER OVERLAY' : 'chip');
}, [box.x + box.width/2, box.y + box.height/2]));

// click the centre with the mouse and see what happens
await page.mouse.click(box.x + box.width/2, box.y + box.height/2);
await page.waitForTimeout(1200);
const dlg = await page.evaluate(() => {
  const d = document.querySelector('[role="dialog"], [role="alertdialog"]');
  return d ? d.innerText.replace(/\n/g,' | ').slice(0,200) : 'NO DIALOG';
});
console.log('after clicking chip centre ->', dlg);
await app.shot('ides-chip-centre-click');
// escape anything open
await page.keyboard.press('Escape');
await page.waitForTimeout(500);
// now click the left edge of the chip
await page.mouse.click(box.x + 12, box.y + box.height/2);
await page.waitForTimeout(1200);
console.log('after clicking chip left edge ->', await page.evaluate(() => {
  const d = document.querySelector('[role="dialog"], [role="alertdialog"]');
  return d ? d.innerText.replace(/\n/g,' | ').slice(0,200) : 'NO DIALOG';
}));
await page.keyboard.press('Escape');
await app.done();
