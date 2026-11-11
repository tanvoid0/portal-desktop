import { open } from './app.mjs';
const app = await open('devices');
await app.page.getByText('Toggle theme', { exact: true }).click();
await app.page.waitForTimeout(200);
await app.page.getByRole('menuitem', { name: 'Dark' }).click();
await app.page.waitForTimeout(300);
await app.done();
