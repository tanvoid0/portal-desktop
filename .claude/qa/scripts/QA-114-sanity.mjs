import { open } from './app.mjs';

const app = await open('devices');
app.page.on('console', (m) => console.log('[page]', m.text()));
await app.page.waitForTimeout(500);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);

await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  const orig = internals.invoke.bind(internals);
  // @ts-ignore
  window.__qa_orig_invoke = orig;
  internals.invoke = (cmd, ...args) => {
    console.log('QA intercept saw cmd:', cmd);
    if (cmd === 'list_devices') return Promise.reject(new Error('QA-114 injected failure'));
    return orig(cmd, ...args);
  };
});

// sanity: call list_devices ourselves right now through the SAME path the app uses
const sanity = await app.page.evaluate(async () => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  try {
    await internals.invoke('list_devices');
    return 'resolved (override NOT hit)';
  } catch (e) {
    return 'rejected (override hit): ' + String(e);
  }
});
console.log('sanity check:', sanity);

console.log('waiting 12s for the background poll...');
await app.page.waitForTimeout(12000);

// restore + cleanup
await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  // @ts-ignore
  internals.invoke = window.__qa_orig_invoke;
});
const devices = await app.page.evaluate(async () => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  return await invoke('list_devices');
});
const minted = devices
  .filter((d) => d.approval_type === 'until_revoked' && d.expires_at !== null)
  .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))[0];
if (minted) {
  const res = await app.page.evaluate(async (id) => {
    // @ts-ignore
    const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
    try {
      await invoke('revoke_device', { deviceId: id });
      return 'ok';
    } catch (e) {
      return 'err: ' + String(e);
    }
  }, minted.device_id);
  console.log('cleanup revoke_device:', minted.device_id, '->', res);
}

await app.done();
