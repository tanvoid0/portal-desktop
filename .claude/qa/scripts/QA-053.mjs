// Env "Add variable": where does the new row appear / does focus move? (nothing is saved)
import { open } from './app.mjs';
const app = await open('utilities/environment');
const p = app.page;
await p.waitForTimeout(1500);
const before = await p.locator('input').count();
await p.getByRole('button',{name:/Add variable/}).click(); await p.waitForTimeout(600);
const ae = await p.evaluate(()=>{const a=document.activeElement;return a.tagName+' ph='+a.placeholder+' val='+a.value});
const rows = await p.locator('input').count();
const first = await p.locator('input').nth(1).inputValue();
const lastEmpty = await p.evaluate(()=>{const i=[...document.querySelectorAll('input')];const idx=i.findIndex(x=>x.value===''&&x.placeholder!=='' && !x.placeholder.startsWith('Search'));return idx+'/'+i.length});
console.log({before, rows, activeElement: ae, firstRowName: first, firstEmptyInputIdx: lastEmpty});
await app.shot('QA-053-before-add-variable-offscreen');
// leave with unsaved change: sidebar link
await p.getByRole('link',{name:'Terminal'}).first().click(); await p.waitForTimeout(1200);
console.log('after nav url', p.url(), 'dialogs', await p.locator('[role=alertdialog],[role=dialog]').count());
await app.done();
