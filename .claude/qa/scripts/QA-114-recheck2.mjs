// QA-114 re-check, instrumented: the first pass saw no PageError and the QR
// stayed visible during fault injection, which contradicts the filed
// description. This adds a call counter to confirm the monkeypatched
// list_devices rejection is actually being hit by the background poll.
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);
const qrUrl = await app.page.locator('code').first().innerText();
console.log('qr_url:', qrUrl);

await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  const orig = internals.invoke.bind(internals);
  // @ts-ignore
  window.__qa_orig_invoke = orig;
  // @ts-ignore
  window.__qa_list_devices_calls = 0;
  // @ts-ignore
  window.__qa_rejected_calls = 0;
  internals.invoke = (cmd, ...args) => {
    if (cmd === 'list_devices') {
      // @ts-ignore
      window.__qa_list_devices_calls++;
      // @ts-ignore
      window.__qa_rejected_calls++;
      return Promise.reject(new Error('QA-114 injected failure'));
    }
    return orig(cmd, ...args);
  };
});

for (let i = 0; i < 6; i++) {
  await app.page.waitForTimeout(1000);
  const calls = await app.page.evaluate(() => (window).__qa_list_devices_calls);
  const bodyHasError = (await app.page.evaluate(() => document.body.innerText)).includes(
    'Failed to load devices'
  );
  const qrVisible = await app.page.locator('img[alt="Pairing QR code"]').isVisible();
  console.log(`t=${i + 1}s calls=${calls} pageErrorShown=${bodyHasError} qrVisible=${qrVisible}`);
}

await app.shot('QA-114-recheck2-state');

// restore + cleanup
await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  // @ts-ignore
  internals.invoke = window.__qa_orig_invoke;
});
await app.page.waitForTimeout(2000);

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
