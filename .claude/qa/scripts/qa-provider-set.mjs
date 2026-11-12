import { open } from './app.mjs';
const app = await open('ai/coder'); const p = app.page; await p.waitForTimeout(2500);
await p.locator('button').filter({ hasText: 'Provider' }).first().click();
await p.getByRole('option', { name: 'Claude Code (CLI)' }).or(p.getByText('Claude Code (CLI)').last()).first().click();
await p.waitForTimeout(1500);
console.log('stored provider:', await p.evaluate(()=>localStorage.getItem('portal.coder.backendProvider')));
console.log(await app.shot('sweep-doctor-coder-provider-cli-selected-dev'));
await app.done();
