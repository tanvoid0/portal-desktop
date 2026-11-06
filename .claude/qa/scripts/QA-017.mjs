// QA-017: a coder session created in a non-registered folder is counted ("1 session") but never listed.
// Precondition: a coder session exists whose workspace is .claude/qa/fixtures/QA coder fixture (not a registered project).
import { open } from './app.mjs';
const app = await open('ai/coder');
const p = app.page;
await p.waitForTimeout(2000);
await p.getByPlaceholder('Search sessions...').fill('list the files');
await p.waitForTimeout(1200);
console.log('header count:', (await p.getByText(/^\d+ sessions?$/).first().innerText()));
const side = p.locator('div', { has: p.getByText('WORKSPACES') }).last();
const txt = await side.innerText();
console.log('"list the files" visible in list:', /list the files/i.test(txt.replace('list the files','')) );
console.log('QA coder fixture folder in list:', /QA coder fixture/.test(txt));
console.log('"No sessions" rows:', (txt.match(/No sessions/g) || []).length);
await app.shot('QA-017-before-session-not-listed');
await app.done();
