// QA-115 re-check, tighter sampling: the 10x1000ms poll in the first recheck
// pass missed the flicker (local invoke round trip is sub-100ms). Samples the
// Refresh button's `disabled` attribute every 50ms for 9s, in-page (no
// Playwright round-trip latency), while a pairing QR is on screen so
// PairDeviceCard's 3s poll is active.
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(500);

const result = await app.page.evaluate(async () => {
  const btn = Array.from(document.querySelectorAll('button')).find((b) =>
    b.textContent?.includes('Refresh')
  );
  if (!btn) return { error: 'refresh button not found' };
  const samples = [];
  const start = performance.now();
  while (performance.now() - start < 9000) {
    samples.push({ t: Math.round(performance.now() - start), busy: btn.disabled });
    await new Promise((r) => setTimeout(r, 50));
  }
  let transitions = 0;
  for (let i = 1; i < samples.length; i++) {
    if (samples[i].busy && !samples[i - 1].busy) transitions++;
  }
  return { transitions, busySamples: samples.filter((s) => s.busy).map((s) => s.t) };
});
console.log('result:', JSON.stringify(result, null, 2));
console.log('QA-115 still reproduces (busy toggled with no user click):', result.transitions > 0);

// cleanup: revoke the fixture this script minted
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
