// Post-fix verification for QA-115 (CDP :9223). Generates a QR to start the
// 3s background poll, then samples the Refresh button's `disabled` attribute
// every 100ms for ~15s with no user click. Fixed behavior: button stays
// enabled throughout (bound to local `refreshing` flag now, not
// devicesQ.isFetching).
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function rawDevices() {
  return app.page.evaluate(async () => {
    // @ts-ignore
    const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
    return await invoke('list_devices');
  });
}

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);

const devices = await rawDevices();
const minted = devices
  .filter((d) => d.approval_type === 'until_revoked' && d.expires_at !== null)
  .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))[0];
if (!minted) throw new Error('could not find minted QR row');
console.log('minted fixture device_id:', minted.device_id);

const refreshBtn = app.page.locator('button').filter({ hasText: 'Refresh' });
let flickerCount = 0;
let samples = 0;
const start = Date.now();
while (Date.now() - start < 15000) {
  const isBusy = (await refreshBtn.getAttribute('disabled')) !== null;
  samples++;
  if (isBusy) flickerCount++;
  await app.page.waitForTimeout(100);
}
console.log(`samples: ${samples}, busy samples (unwanted, no click happened): ${flickerCount}`);
console.log('QA-115 FIXED (button never went busy on its own):', flickerCount === 0);

console.log('\n--- cleanup ---');
const cleanup = await app.page.evaluate(async (id) => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  try {
    await invoke('revoke_device', { deviceId: id });
    return 'ok';
  } catch (e) {
    return 'err: ' + String(e);
  }
}, minted.device_id);
console.log('revoke_device result:', cleanup);

await app.done();
