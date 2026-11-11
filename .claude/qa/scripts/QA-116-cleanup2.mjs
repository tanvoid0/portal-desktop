// Revoke exactly the 5 QR-pairing fixture rows this verification created
// (3 of them are unscanned/expired QR grants, invisible in the UI per the
// known QA-117 gap, so the UI "Revoke" button can't reach them -- using the
// same single-device-scoped `revoke_device` command the UI's Revoke button
// itself calls, not a bulk/reset operation).
import { open } from './app.mjs';

const ids = [
  '3337ed33-918e-4294-93e9-1952b731bdfd',
  '8352190b-49a3-4987-be2d-17f243fb415c',
  'b6fc6901-dca3-4574-823f-3ec5aacb5e03',
  'c894363b-077f-4220-a3a3-14dcddf028eb',
  '92fc3e54-f9d0-4dad-80e5-33722f8b0d89',
];

const app = await open('devices');
await app.page.waitForTimeout(500);

for (const device_id of ids) {
  const res = await app.page.evaluate(async (id) => {
    // @ts-ignore
    const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
    try {
      await invoke('revoke_device', { deviceId: id });
      return 'ok';
    } catch (e) {
      return 'err: ' + String(e);
    }
  }, device_id);
  console.log(device_id, '->', res);
}

const raw = await app.page.evaluate(async () => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  return await invoke('list_devices');
});
console.log('remaining rows:', JSON.stringify(raw.map((r) => ({ id: r.device_id, name: r.device_name, approved: r.approved, type: r.approval_type })), null, 2));

await app.done();
