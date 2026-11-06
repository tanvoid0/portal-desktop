// Calls ai_platform_health in the running app and prints the result.
import { chromium } from '@playwright/test';
const b = await chromium.connectOverCDP('http://127.0.0.1:9222', { timeout: 5000 });
const page = b.contexts()[0].pages()[0];
await page.goto('http://localhost:1420/');
await page.waitForTimeout(2000);
const out = await page.evaluate(async () => {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) return { error: 'no tauri invoke on this page' };
  try {
    return { health: await invoke('ai_platform_health') };
  } catch (e) {
    return { error: String(e) };
  }
});
console.log(JSON.stringify(out, null, 1));
await b.close();
