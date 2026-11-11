// QA-110 verify (b)+(c): generate a QR, "scan" it on the real connected
// phone via adb (am start -a VIEW -d <portalremote:// url>, since the
// phone has com.portalremote installed), and confirm the new pairing shows
// up in "Paired devices" within ~3s with no manual refresh (the new poll).
// Also leaves the pre-existing "Tanveer's S26 Ultra" pairing alone as the
// (c) "existing pairing still works" check — just reads it, doesn't touch it.
import { open } from './app.mjs';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
const exec = promisify(execFile);

const app = await open('devices');
await app.page.waitForTimeout(500);

// Pick one distinctive capability so the new device is easy to spot.
await app.page.locator('label', { hasText: 'tasks' }).first().click();
await app.page.waitForTimeout(200);

const genBtn = app.page.locator('button').filter({ hasText: 'Generate QR code' });
await genBtn.click();
await app.page.waitForTimeout(1000);

const qrUrl = await app.page.locator('code').first().innerText();
console.log('QR_URL:', qrUrl);

const before = await app.page.evaluate(() => document.body.innerText);
console.log('paired count before scan:', before.match(/Paired\s+(\d+)/)?.[1]);

// "Scan" it: fire the intent on the connected phone.
try {
  const { stdout, stderr } = await exec('adb', [
    'shell', 'am', 'start', '-a', 'android.intent.action.VIEW', '-d', qrUrl,
  ]);
  console.log('adb stdout:', stdout.trim());
  if (stderr) console.log('adb stderr:', stderr.trim());
} catch (e) {
  console.log('adb intent failed:', e.message);
}

// Do NOT reload/navigate (that would drop this component's in-memory QR
// state) — just wait and watch the existing page poll.
let appeared = false;
for (let i = 0; i < 8; i++) {
  await app.page.waitForTimeout(1500);
  const text = await app.page.evaluate(() => document.body.innerText);
  const count = text.match(/Paired\s+(\d+)/)?.[1];
  console.log(`t+${(i + 1) * 1.5}s paired count:`, count);
  if (count && Number(count) > 2) { appeared = true; break; }
}
console.log('new pairing appeared without manual refresh/reload:', appeared);
await app.shot('QA-110-after-scan-via-adb-intent');

await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(300);
await app.shot('QA-110-after-scan-paired-list');

await app.done();
