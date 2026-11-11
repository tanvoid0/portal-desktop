// QA-116 part (b), take 3: same as scan2 but the phone curl runs via a single
// pre-quoted remote shell string (adb flattens execFile argv with spaces into
// one command line server-side, which silently drops -H's value if it isn't
// already quoted for the REMOTE shell -- scan2's execFile array form hit
// exactly that gotcha and sent no Authorization header at all, confirmed via
// `adb shell curl -v`). Generates a QR, fires the phone-side curl correctly
// quoted, and watches the already-open page (no reload) for the poll to pick
// it up.
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

const remoteCmd = `curl -s -o /dev/null -w '%{http_code}' -H 'Authorization: Bearer ${token}' http://${host}:${port}/health`;
const t0 = Date.now();
const { stdout } = await run('adb', ['-s', '192.168.0.235:45867', 'shell', remoteCmd]);
console.log('phone curl http_code:', stdout.trim());

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
console.log(
  'RESULT picked up without manual refresh within ~3-6s window:',
  pickedUpAt !== null && pickedUpAt <= 6000
);
await app.shot('QA-116-reverify-scan3-picked-up');

await app.done();
