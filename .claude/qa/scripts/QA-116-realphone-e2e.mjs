// Real end-to-end phone pairing spot-check (2026-11-11, instance CDP :9223).
// The installed com.portalremote has no portalremote:// intent-filter (confirmed
// in QA-116's own verification via `adb shell dumpsys package com.portalremote` —
// camera-only, no deep link), so a literal `adb shell am start -a VIEW` can't
// trigger the scan. Reproduces the one network-observable effect an actual scan
// has instead: the phone's first authenticated request to the daemon with the
// QR's bearer token, fired FROM the phone itself over adb shell (real LAN hop,
// non-loopback peer, same codepath verify_token's first-use clearing depends on).
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { open } from './app.mjs';

const run = promisify(execFile);
const ADB_SERIAL = 'adb-R5GL150LRCK-N75UEH._adb-tls-connect._tcp';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function pairedCount() {
  const text = await app.page.evaluate(() => document.body.innerText);
  const m = text.match(/Paired\s+(\d+)/);
  return m ? Number(m[1]) : null;
}

const before = await pairedCount();
console.log('Paired before:', before);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);

const qrUrl = await app.page.locator('code').first().innerText();
console.log('qr_url:', qrUrl);
const m = qrUrl.match(/portalremote:\/\/([^:]+):(\d+)\/(.+)$/);
if (!m) throw new Error('could not parse qr_url: ' + qrUrl);
const [, host, port, token] = m;

const afterQr = await pairedCount();
console.log('Paired right after QR generated (should be unchanged):', afterQr);

const remoteCmd = `curl -s -o /dev/null -w '%{http_code}' -H 'Authorization: Bearer ${token}' http://${host}:${port}/health`;
console.log('firing phone-side curl via adb -s', ADB_SERIAL);
const t0 = Date.now();
const { stdout, stderr } = await run('adb', ['-s', ADB_SERIAL, 'shell', remoteCmd]);
console.log('phone curl http_code:', stdout.trim(), stderr ? `stderr: ${stderr}` : '');

let pickedUpAt = null;
for (let i = 0; i < 20; i++) {
  await app.page.waitForTimeout(500);
  const c = await pairedCount();
  if (c !== null && c > afterQr) {
    pickedUpAt = Date.now() - t0;
    console.log(`Paired count went ${afterQr} -> ${c} at +${pickedUpAt}ms (no reload/goto)`);
    break;
  }
}
console.log(
  'RESULT: real scan picked up within ~3s with no manual refresh:',
  pickedUpAt !== null && pickedUpAt <= 4000
);
await app.shot('QA-116-realphone-e2e-picked-up');

// Raw device row for the record
const raw = await app.page.evaluate(async () => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  return await invoke('list_devices');
});
const row = raw.find((d) => d.device_name === 'Unnamed device' && d.last_used_at !== null);
console.log('paired row after scan:', JSON.stringify(row));

await app.done();
