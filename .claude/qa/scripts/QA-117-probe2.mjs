import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);
await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(1200);

const regexCount = await app.page.locator('button').filter({ hasText: /^Cancel$/ }).count();
const plainCount = await app.page.locator('button').filter({ hasText: 'Cancel' }).count();
console.log('regex count:', regexCount, 'plain count:', plainCount);

const devices = await app.page.evaluate(async () => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  return await invoke('list_devices');
});
const minted = devices
  .filter((d) => d.approval_type === 'until_revoked' && d.expires_at !== null)
  .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))[0];
if (minted) {
  await app.page.evaluate(async (id) => {
    // @ts-ignore
    const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
    await invoke('revoke_device', { deviceId: id });
  }, minted.device_id);
  console.log('cleaned up', minted.device_id);
}
await app.done();
