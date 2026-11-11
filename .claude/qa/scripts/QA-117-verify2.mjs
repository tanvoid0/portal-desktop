import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(1000);

const texts = await app.page.locator('button').allInnerTexts();
console.log('buttons after generate:', JSON.stringify(texts));

// scroll main pane down to make sure nothing is just below the fold
await app.page.evaluate(() => {
  const main = document.querySelector('main') || document.scrollingElement;
  main?.scrollTo?.(0, main.scrollHeight);
});
await app.page.waitForTimeout(300);
await app.shot('QA-117-verify2-after-generate-scrolled');

const texts2 = await app.page.locator('button').allInnerTexts();
console.log('buttons after scroll:', JSON.stringify(texts2));

// cleanup: find minted and revoke
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
