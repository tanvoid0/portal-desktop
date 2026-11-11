import { open } from './app.mjs';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
const exec = promisify(execFile);

const app = await open('devices');
await app.page.waitForTimeout(500);

await app.page.locator('label', { hasText: 'tasks' }).first().click();
await app.page.waitForTimeout(150);
await app.page.locator('button').filter({ hasText: 'Generate QR code' }).click();
await app.page.waitForTimeout(800);

const qrUrl = await app.page.locator('code').first().innerText();
console.log('QR_URL:', qrUrl);
const m = qrUrl.match(/^portalremote:\/\/([^:]+):(\d+)\/(.+)$/);
console.log('parsed:', m && { host: m[1], port: m[2], token: m[3] });

const before = await app.page.evaluate(() => document.body.innerText.match(/Paired\s+(\d+)/)?.[1]);
console.log('paired before:', before);

const t0 = Date.now();
const { stdout, stderr } = await exec('adb', [
  'shell', 'curl', '-s', '-o', '/dev/null', '-w', '%{http_code}',
  '-H', `Authorization: Bearer ${m[3]}`,
  `http://${m[1]}:${m[2]}/health`,
]);
console.log(`curl from phone (t+${Date.now() - t0}ms): http_code=${stdout.trim()}`, stderr.trim());

let appeared = false;
for (let i = 0; i < 6; i++) {
  await app.page.waitForTimeout(1500);
  const count = await app.page.evaluate(() => document.body.innerText.match(/Paired\s+(\d+)/)?.[1]);
  console.log(`t+${(i + 1) * 1.5}s paired:`, count);
  if (count && Number(count) > Number(before)) { appeared = true; break; }
}
console.log('appeared without manual refresh:', appeared);
await app.shot('QA-110-verify-b2-result');
await app.page.mouse.wheel(0, 1200);
await app.page.waitForTimeout(200);
await app.shot('QA-110-verify-b2-paired-list');
await app.done();
