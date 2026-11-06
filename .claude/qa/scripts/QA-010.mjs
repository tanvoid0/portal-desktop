// QA-010: breadcrumb shows only "Home" on /settings/general when reached via the sidebar footer gear.
import { open } from './app.mjs';
const app = await open('');
const p = app.page;
const crumb = () => p.locator('header').first().innerText().then(t => t.replace(/\s+/g, ' ').trim());
await p.getByRole('button', { name: 'Settings' }).last().click();
await p.waitForTimeout(1500);
const c = await crumb();
console.log('url', new URL(p.url()).pathname, '| header text:', c, c.includes('Settings') ? 'ok' : 'FAIL');
await app.shot('QA-010-before-settings-crumb-home');
await p.goto(new URL('/settings/general', p.url()).href); await p.waitForTimeout(1200);
console.log('direct load header text:', await crumb());
await app.done();
