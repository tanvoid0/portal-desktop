import { open } from './app.mjs';
import { exec as execCb } from 'node:child_process';
import { promisify } from 'node:util';
const exec = promisify(execCb);

const app = await open('devices');
await app.page.waitForTimeout(500);

await app.page.locator('label', { hasText: 'coder' }).first().click();
await app.page.waitForTimeout(150);
await app.page.locator('button').filter({ hasText: /Generate QR code|Regenerate QR code/ }).click();
await app.page.waitForTimeout(800);

const qrUrl = await app.page.locator('code').first().innerText();
const m = qrUrl.match(/^portalremote:\/\/([^:]+):(\d+)\/(.+)$/);
console.log('token:', m[3]);

// Baseline read right after generating (this single refetch is expected,
// from generate()'s own onPaired() call) — then we watch for the NEXT
// change to arrive via the 3s poll alone, with no further navigation.
await app.page.waitForTimeout(300);
const before = await app.page.evaluate(() => document.body.innerText.match(/Paired\s+(\d+)/)?.[1]);
console.log('paired baseline:', before);

const cmd = `adb shell "curl -s -o /dev/null -w '%{http_code}\\n' -H 'Authorization: Bearer ${m[3]}' http://${m[1]}:${m[2]}/health"`;
const t0 = Date.now();
const { stdout, stderr } = await exec(cmd);
console.log(`curl done at t+${Date.now() - t0}ms, http_code=${stdout.trim()}`, stderr?.trim() || '');
const tScan = Date.now();

let appeared = false;
for (let i = 0; i < 10; i++) {
  await app.page.waitForTimeout(1000);
  const count = await app.page.evaluate(() => document.body.innerText.match(/Paired\s+(\d+)/)?.[1]);
  console.log(`t+${Date.now() - tScan}ms since scan, paired:`, count);
  if (count && Number(count) > Number(before)) { appeared = true; console.log('>>> picked up after', Date.now() - tScan, 'ms, no manual refresh'); break; }
}
console.log('RESULT appeared via poll alone:', appeared);
await app.shot('QA-110-verify-b3-result');
await app.done();
