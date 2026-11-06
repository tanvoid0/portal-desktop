// Repro: Add Credential -> Create fails (fake value only). Fixture is never created because the call is rejected.
import { open } from './app.mjs';
const app = await open('credentials');
const p = app.page;
await p.waitForTimeout(1000);
await p.getByRole('button', { name: 'Add Credential' }).click();
const d = p.locator('[role=dialog]');
await d.locator('#name').fill('QA credential');
await d.locator('#value').fill('qa-fake-value');
await d.getByRole('button', { name: 'Create' }).click();
await p.waitForTimeout(1500);
console.log('dialog still open:', await d.count(), '(expected 0)');
console.log('toast:', await p.locator('[data-sonner-toast]').allInnerTexts());
await d.screenshot({ path: new URL('../shots/QA-060-before-create-credential-fails.png', import.meta.url).pathname.replace(/^\//, '') });
await p.getByRole('button', { name: 'Cancel' }).click();
await app.done();
