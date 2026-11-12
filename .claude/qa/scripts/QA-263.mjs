// #263: /pair/request must persist a pending row, show on Devices, and complete on Allow.
import { open } from './app.mjs';
import { execSync } from 'node:child_process';
const app = await open('devices');
const p = app.page;
await p.waitForTimeout(2500);
const text = await p.locator('body').innerText();
console.log('1. pending card shows QA263E2E:', text.includes('QA263E2E'));
await app.shot('QA-263-pending');
const list = await p.evaluate(() => window.__TAURI_INTERNALS__.invoke('list_devices', {}).catch((e) => String(e)));
const row = Array.isArray(list) ? list.find((d) => d.device_name === 'QA263E2E') : null;
console.log('2. list_devices has pending row:', !!row, row?.approval_type);
if (row) {
  const r = await p.evaluate((id) => window.__TAURI_INTERNALS__.invoke('approve_device', { request: { device_id: id, approval_type: 'until_revoked', capabilities: [] } }), row.device_id);
  console.log('3. approve:', JSON.stringify(r));
}
await app.done();
