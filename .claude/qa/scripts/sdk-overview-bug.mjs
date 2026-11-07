import { open } from './i2.mjs';
const app = await open('sdk/managers', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(9000);
console.log('managers page loaded, rows present:', (await page.innerText('body')).includes('Conda'));
// client-side nav to Overview via the SDK sidebar
await page.getByRole('link', { name: /Overview/ }).last().click();
await page.waitForTimeout(4000);
console.log('url:', page.url());
let b = await page.innerText('body');
console.log('after client nav -> loading:', b.includes('Loading SDK data'), '| quickNav:', b.includes('Quick Navigation'), '| stats:', (b.match(/Managers installed[\s\S]{0,70}/)||[''])[0].replace(/\n/g,' | '));
await app.shot('sdk-overview-via-client-nav');
// now hard reload /sdk
await app.go('sdk');
await page.waitForTimeout(12000);
b = await page.innerText('body');
console.log('after hard load /sdk -> loading:', b.includes('Loading SDK data'), '| quickNav:', b.includes('Quick Navigation'));
await app.shot('sdk-overview-hard-load-stuck');
await app.done();
