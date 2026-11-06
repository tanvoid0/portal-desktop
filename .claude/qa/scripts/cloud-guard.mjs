import { open } from './app.mjs';
const app = await open('cloud/workloads');
const p = app.page;
await p.waitForSelector('text=Cluster Connection Required', {timeout:15000});
// retry
const retry = p.getByRole('button', { name: /^Retry/ });
await retry.click();
const seen=[];
for (let i=0;i<10;i++){ seen.push((await p.getByRole('button',{name:/Retry|Loading/}).first().innerText())); await p.waitForTimeout(150);}
console.log('retry label over time', seen.join('|'));
// tool refresh
await p.getByRole('button',{name:'Refresh tools'}).click();
await p.waitForTimeout(100);
console.log('after refresh text:', (await p.locator('text=Detecting').count()));
// toasts?
console.log('toasts:', await p.locator('[data-sonner-toast]').allInnerTexts());
// keyboard: tab order
await p.keyboard.press('Tab');
const focus=[];
for (let i=0;i<4;i++){ focus.push(await p.evaluate(()=>document.activeElement?.innerText?.slice(0,30)||document.activeElement?.tagName)); await p.keyboard.press('Tab'); }
console.log('focus', focus);
await p.emulateMedia({colorScheme:'dark'});
await p.waitForTimeout(500);
console.log(await app.shot('sweep-cloud-not-connected-dark'));
await p.emulateMedia({colorScheme:'light'});
await p.setViewportSize({width:1024,height:600});
await p.waitForTimeout(500);
console.log(await app.shot('sweep-cloud-not-connected-1024x600'));
const sw = await p.evaluate(()=>({sw:document.documentElement.scrollWidth, cw:document.documentElement.clientWidth}));
console.log('scrollW', sw);
await p.setViewportSize({width:1280,height:800});
await app.done();
