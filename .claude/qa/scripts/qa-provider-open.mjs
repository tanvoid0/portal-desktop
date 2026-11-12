import { open } from './app.mjs';
const app = await open('ai/coder'); const p = app.page; await p.waitForTimeout(2500);
console.log('stored provider:', await p.evaluate(()=>localStorage.getItem('portal.coder.backendProvider')));
await p.locator('button').filter({ hasText: 'Provider' }).first().click();
await p.waitForTimeout(800);
console.log(await p.locator('[role="listbox"], [role="menu"]').allInnerTexts());
console.log(await app.shot('sweep-doctor-coder-provider-options-dev'));
await p.keyboard.press('Escape'); await app.done();
