import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

const trigger = app.page.getByText('Toggle theme', { exact: true });
console.log('toggle theme trigger count:', await trigger.count());
await trigger.click();
await app.page.waitForTimeout(200);
const lightItem = app.page.getByRole('menuitem', { name: 'Light' });
console.log('light menuitem count:', await lightItem.count());
await lightItem.click();
await app.page.waitForTimeout(400);
await app.shot('sweep-devices-light');

await app.done();
