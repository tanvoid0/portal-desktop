// Sweep: Devices page — scroll to paired devices / history, dump device data.
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

// Scroll main pane to bottom to capture paired devices + history
await app.page.mouse.wheel(0, 2000);
await app.page.waitForTimeout(300);
await app.shot('sweep-devices-paired-scrolled');

// Dump paired device card text content
const cards = await app.page.locator('text=Paired devices').locator('xpath=following-sibling::*[1]').innerText().catch(() => 'N/A');
console.log('Paired devices block text:\n', cards);

await app.done();
