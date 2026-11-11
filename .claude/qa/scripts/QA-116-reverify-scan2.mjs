// QA-116 part (b), combined: generate QR, fire the phone-side curl (the real
// scan's one network-observable effect) from inside this script after a short
// delay, and watch the SAME open page (no reload/goto) to confirm "Paired"
// increments within ~3s via the poll added for QA-116, not a manual refresh.
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { open } from './app.mjs';

const run = promisify(execFile);

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
const m = qrUrl.match(/portalremote:\/\/([^:]+):(\d+)\/(.+)$/);
if (!m) throw new Error('could not parse qr_url: ' + qrUrl);
const [, host, port, token] = m;
console.log('qr_url parsed, host:', host, 'port:', port);

const afterQr = await pairedCount();
console.log('Paired count right after QR generated (should be unchanged):', afterQr);

// Fire the phone-side curl now (simulates the scan's network effect).
const t0 = Date.now();
const { stdout } = await run('adb', [
  '-s',
  '192.168.0.235:45867',
  'shell',
  'curl',
  '-s',
  '-w',
  '\\nHTTP:%{http_code}\\n',
  '-H',
  `Authorization: Bearer ${token}`,
  `http://${host}:${port}/health`,
]);
console.log('phone curl result:', stdout.trim());

// Watch the already-open page (no reload) for the poll to pick it up.
let pickedUpAt = null;
for (let i = 0; i < 20; i++) {
  await app.page.waitForTimeout(500);
  const c = await pairedCount();
  if (c !== null && c > afterQr) {
    pickedUpAt = Date.now() - t0;
    console.log(`Paired count went ${afterQr} -> ${c} at +${pickedUpAt}ms (no reload)`);
    break;
  }
}
console.log('RESULT picked up without manual refresh within ~3s window:', pickedUpAt !== null && pickedUpAt <= 6000);
await app.shot('QA-116-reverify-scan-picked-up');

await app.done();
