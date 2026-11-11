// QA-117 part B: generate a QR, scan it for real (phone-side curl over adb,
// same technique as QA-116-realphone-e2e.mjs), confirm the card auto-clears
// to "Generate QR code" instead of leaving a stale Cancel button on an
// already-paired device.
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { open } from './app.mjs';

const run = promisify(execFile);
const ADB_SERIAL = 'adb-R5GL150LRCK-N75UEH._adb-tls-connect._tcp';

const app = await open('devices');
await app.page.waitForTimeout(500);

await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(1000);

const cancelVisibleBefore = await app.page.locator('button').filter({ hasText: 'Cancel' }).isVisible();
console.log('Cancel button visible before scan:', cancelVisibleBefore);

const qrUrl = await app.page.locator('code').first().innerText();
const m = qrUrl.match(/portalremote:\/\/([^:]+):(\d+)\/(.+)$/);
if (!m) throw new Error('could not parse qr_url: ' + qrUrl);
const [, host, port, token] = m;

const remoteCmd = `curl -s -o /dev/null -w '%{http_code}' -H 'Authorization: Bearer ${token}' http://${host}:${port}/health`;
console.log('firing phone-side curl via adb -s', ADB_SERIAL);
const { stdout } = await run('adb', ['-s', ADB_SERIAL, 'shell', remoteCmd]);
console.log('phone curl http_code:', stdout.trim());

let clearedAt = null;
const start = Date.now();
for (let i = 0; i < 20; i++) {
  await app.page.waitForTimeout(500);
  const generateShown = await app.page.locator('button').filter({ hasText: 'Generate QR code' }).isVisible();
  const cancelShown = await app.page.locator('button').filter({ hasText: 'Cancel' }).isVisible();
  if (generateShown && !cancelShown) {
    clearedAt = Date.now() - start;
    break;
  }
}
console.log('Card auto-cleared back to "Generate QR code" (no stale Cancel) within', clearedAt, 'ms');
await app.shot('QA-117-after-scan-autoclear');

console.log('\nQA-117 Part B PASS (auto-clear on real scan):', clearedAt !== null);

await app.done();
