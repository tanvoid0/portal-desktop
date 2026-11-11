// Re-verify QA-116 part (b) on instance-2: generate a QR, read its qrUrl text
// (shown in-app under the image), then simulate the one network-observable
// effect of a real scan -- the phone's first authenticated request bearing
// the QR token -- by running curl ON THE PHONE over adb (same technique the
// prior verification used; the companion app has no portalremote:// intent
// filter, confirmed via `adb shell dumpsys package com.portalremote`, so a
// deep-link VIEW intent can't trigger a scan and there's no way to feed a
// real camera a displayed QR through adb). Confirms the 3s poll added for
// QA-116 picks up a real LAN-authenticated first-use with no manual refresh.
import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

async function pairedCount() {
  const text = await app.page.evaluate(() => document.body.innerText);
  const m = text.match(/Paired\s+(\d+)/);
  return m ? Number(m[1]) : null;
}

const before = await pairedCount();
console.log('Paired count before QR:', before);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(1000);

const qrUrl = await app.page.locator('code').first().innerText();
console.log('qr_url:', qrUrl);
// portalremote://<host>:<port>/<token>
const m = qrUrl.match(/portalremote:\/\/([^:]+):(\d+)\/(.+)$/);
if (!m) throw new Error('could not parse qr_url: ' + qrUrl);
const [, host, port, token] = m;
console.log('parsed -> host:', host, 'port:', port, 'token:', token);

console.log('RESULT: this script only prepares the QR; run the printed adb curl command');
console.log(
  `ADB_CMD: adb -s 192.168.0.235:45867 shell curl -s -o /dev/null -w '%{http_code}' -H "Authorization: Bearer ${token}" http://${host}:${port}/health`
);

await app.shot('QA-116-reverify-scan-qr-up');
await app.done();
