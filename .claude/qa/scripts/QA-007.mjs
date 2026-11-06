// QA-007: header "Go forward" is never enabled after "Go back".
import { open } from './app.mjs';
const app = await open('settings/general');
const p = app.page;
await p.getByRole('link', { name: 'Terminal' }).first().click();
await p.waitForTimeout(800);
console.log('on', new URL(p.url()).pathname);
await p.getByRole('button', { name: 'Go back' }).click();
await p.waitForTimeout(1000);
const fwdDisabled = await p.getByRole('button', { name: 'Go forward' }).isDisabled();
console.log('on', new URL(p.url()).pathname, '| Go forward disabled:', fwdDisabled, fwdDisabled ? 'FAIL' : 'ok');
await app.shot('QA-007-before-forward-disabled');
await p.evaluate(() => history.forward()); // the browser history itself does have a forward entry
await p.waitForTimeout(800);
console.log('history.forward() ->', new URL(p.url()).pathname);
await app.done();
