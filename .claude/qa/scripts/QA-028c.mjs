// Repro: archiving a document hides it from /documents with no way to find or restore it.
import { open } from './app.mjs';
const app = await open('documents/1');
const p = app.page;
await p.waitForTimeout(800);
await p.getByLabel(/Archived/).click();
await p.getByRole('button', { name: 'Save Document' }).click();
await p.waitForTimeout(1200);
await p.goto(new URL('/documents', p.url()).href);
await p.waitForTimeout(1500);
const txt = await p.locator('main').innerText();
console.log('archived doc listed:', txt.includes('QA fixture doc'), '| archive filter/toggle on page:', /archiv/i.test(txt));
await app.shot('QA-037-before-archived-doc-vanishes');
// restore the fixture
await p.goto(new URL('/documents/1', p.url()).href);
await p.waitForTimeout(1000);
await p.getByLabel(/Archived/).click();
await p.getByRole('button', { name: 'Save Document' }).click();
await p.waitForTimeout(1000);
await app.done();
