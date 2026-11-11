import { chromium } from 'playwright';

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages().find((p) => p.url().includes('/devices'));
await page.bringToFront();

const result = await page.evaluate(async () => {
  const raw = await window.__TAURI_INTERNALS__.invoke('list_devices');
  const ghost = raw.find((d) => d.device_name === "Tanveer's S26 Ultra" && d.approved);
  if (!ghost) return { note: 'no ghost paired row found', raw };
  await window.__TAURI_INTERNALS__.invoke('revoke_device', { deviceId: ghost.device_id });
  return { revoked: ghost.device_id };
});

console.log(JSON.stringify(result, null, 2));
await browser.close();
