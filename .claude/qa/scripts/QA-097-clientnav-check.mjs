import { open } from './app.mjs';

const app = await open('sdk/managers');
await app.page.waitForTimeout(3000);
// Client-side nav to /sdk via the sidebar's Overview link.
const overviewLink = app.page.getByRole('link', { name: /^Overview$/i }).first();
console.log('overview link count:', await overviewLink.count());
await overviewLink.click();
await app.page.waitForTimeout(3000);
const state = await app.page.evaluate(() => ({
  stillLoading: document.body.innerText.includes('Loading SDK data'),
  hasQuickNav: document.body.innerText.includes('Quick Navigation'),
  statsLine: (document.body.innerText.match(/Managers installed[\s\S]{0,60}/) || [null])[0],
}));
console.log('after client nav to /sdk:', JSON.stringify(state));
await app.shot('QA-097-clientnav-check');
await app.done();
