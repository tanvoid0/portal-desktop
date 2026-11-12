import { open } from './app.mjs';
const app = await open('ai/coder'); const p = app.page;
await p.waitForTimeout(3000);
const b = p.locator('button').filter({ hasText: 'Launch Agent Platform' }).first();
console.log('tag', await b.evaluate(e=>e.tagName));
await b.click();
for (let i=0;i<30;i++){ await p.waitForTimeout(3000); if(!(await p.getByText('Agent Platform is not running').count())) {console.log('up after',(i+1)*3,'s');break;} }
console.log(await app.shot('sweep-doctor-coder-after-launch-platform-dev'));
console.log('toasts', await p.locator('[data-sonner-toast]').allInnerTexts());
await app.done();
