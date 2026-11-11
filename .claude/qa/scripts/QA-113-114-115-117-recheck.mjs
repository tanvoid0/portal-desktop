// Re-check pass (2026-11-11) for QA-113/114/115/117 against the live app
// (CDP :9223). Generates one unscanned QR fixture and uses it for all four
// checks, then revokes it via the raw `revoke_device` command (same single-
// device action the UI's own Revoke button calls — the row is invisible in
// the UI by design, per QA-117, so the button isn't reachable).
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function bodyText() {
  return app.page.evaluate(() => document.body.innerText);
}
async function pairedCount() {
  const m = (await bodyText()).match(/Paired\s+(\d+)/);
  return m ? Number(m[1]) : null;
}
async function rawDevices() {
  return app.page.evaluate(async () => {
    // @ts-ignore
    const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
    return await invoke('list_devices');
  });
}

console.log('--- baseline ---');
const pairedBefore = await pairedCount();
console.log('Paired before:', pairedBefore);

console.log('\n--- generate unscanned QR (no capabilities) ---');
await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);
const qrUrl = await app.page.locator('code').first().innerText();
console.log('qr_url:', qrUrl);

const pairedAfterQr = await pairedCount();
console.log('Paired right after QR (QA-116 regression check, should be unchanged):', pairedAfterQr);

const devices = await rawDevices();
const minted = devices
  .filter((d) => d.approval_type === 'until_revoked' && d.expires_at !== null)
  .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))[0];
if (!minted) throw new Error('could not find the freshly minted QR row via list_devices');
console.log('minted row (raw):', JSON.stringify(minted));

console.log('\n--- QA-117: is the unscanned grant reachable anywhere in the UI? ---');
const text = await bodyText();
console.log('page text mentions "Unnamed device":', text.includes('Unnamed device'));
console.log(
  'QA-117 still reproduces (row exists in DB, approved=true, but no UI surface mentions it):',
  minted.approved === true && !text.includes('Unnamed device')
);

console.log('\n--- QA-115: does the Refresh button flicker busy from the background poll? ---');
const refreshBtn = app.page.locator('button').filter({ hasText: 'Refresh' });
let flickerCount = 0;
let wasBusy = await refreshBtn.getAttribute('disabled');
for (let i = 0; i < 10; i++) {
  await app.page.waitForTimeout(1000);
  const isBusy = (await refreshBtn.getAttribute('disabled')) !== null;
  if (isBusy && wasBusy === null) flickerCount++;
  wasBusy = isBusy ? 'true' : null;
}
console.log(`Refresh button went idle->busy ${flickerCount} time(s) over 10s with no user click`);
console.log('QA-115 still reproduces:', flickerCount > 0);

console.log('\n--- QA-114: does a transient list_devices failure wipe the QR card? ---');
const qrVisibleBefore = await app.page.locator('img[alt="Pairing QR code"]').isVisible();
console.log('QR image visible before fault injection:', qrVisibleBefore);

// Monkeypatch invoke in the page (test harness only, not app code) so the next
// list_devices calls reject, simulating one backend hiccup during the 3s poll.
await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  const orig = internals.invoke.bind(internals);
  // @ts-ignore
  window.__qa_orig_invoke = orig;
  internals.invoke = (cmd, ...args) => {
    if (cmd === 'list_devices') return Promise.reject(new Error('QA-114 injected failure'));
    return orig(cmd, ...args);
  };
});

await app.page.waitForTimeout(4500); // >= one 3s poll tick + retry:1 backoff
await app.shot('QA-114-recheck-during-fault');
const textDuringFault = await bodyText();
const qrVisibleDuringFault = await app.page.locator('img[alt="Pairing QR code"]').isVisible();
console.log('Shows "Failed to load devices" during fault:', textDuringFault.includes('Failed to load devices'));
console.log('QR image still visible during fault:', qrVisibleDuringFault);
console.log(
  'QA-114 still reproduces (PageError replaced the whole page, QR card gone):',
  textDuringFault.includes('Failed to load devices') && !qrVisibleDuringFault
);

// restore
await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__ ?? window.__TAURI__?.core;
  // @ts-ignore
  internals.invoke = window.__qa_orig_invoke;
});
await app.page.waitForTimeout(4000);
const textAfterRestore = await bodyText();
console.log('\nRecovered after restore (no more "Failed to load devices"):', !textAfterRestore.includes('Failed to load devices'));
await app.shot('QA-114-recheck-after-restore');

console.log('\n--- cleanup: revoke the fixture row ---');
const cleanupRes = await app.page.evaluate(async (id) => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  try {
    await invoke('revoke_device', { deviceId: id });
    return 'ok';
  } catch (e) {
    return 'err: ' + String(e);
  }
}, minted.device_id);
console.log('revoke_device result:', cleanupRes);
const pairedAfterCleanup = await pairedCount();
console.log('Paired after cleanup (should match baseline):', pairedAfterCleanup, 'baseline was', pairedBefore);

await app.done();
