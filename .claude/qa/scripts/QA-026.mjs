// Repro: /documents/create cannot save. Expect navigation to /documents/<id>.
import { open } from './app.mjs';
const app = await open('documents/create');
const p = app.page;
console.log('card title:', await p.locator('[data-slot=card-title]').first().innerText().catch(() => '?'));
await p.getByPlaceholder('Enter document title...').fill('QA repro doc');
await p.getByRole('button', { name: /^(Save|Create) Document$/ }).click();
await p.waitForTimeout(2000);
console.log('url after save:', p.url(), '(expected /documents/<id>)');
await app.shot('QA-026-before-create-fails');
await app.done();
