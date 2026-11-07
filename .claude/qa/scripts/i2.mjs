// Same as app.mjs but drives dev instance 2 (vite :1421, CDP :9223, own database).
// Instance 1 (:1420/:9222) belongs to the owner and must not be touched.
//
//   node .claude/qa/scripts/i2.mjs logs sweep-logs-i2
import { chromium } from '@playwright/test';
import { existsSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SHOTS = fileURLToPath(new URL('../shots/', import.meta.url));

export async function open(route = '/', { width = 1280, height = 800 } = {}) {
  const browser = await chromium.connectOverCDP('http://127.0.0.1:9223', { timeout: 5000 });
  const page = browser.contexts()[0].pages()[0];
  const errors = [];
  page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
  page.on('pageerror', (e) => errors.push(e.message));

  await page.setViewportSize({ width, height }).catch(() => {});
  await page.goto(new URL(route.replace(/^\/*/, '/'), 'http://localhost:1421').href);
  await page.waitForLoadState('load');
  await page.waitForTimeout(1200);

  return {
    page,
    browser,
    mode: 'tauri-instance-2',
    errors,
    async size(width, height) {
      await page.setViewportSize({ width, height }).catch(() => {});
      await page.waitForTimeout(400);
    },
    async go(r) {
      await page.goto(new URL(r.replace(/^\/*/, '/'), 'http://localhost:1421').href);
      await page.waitForLoadState('load');
      await page.waitForTimeout(1200);
    },
    async shot(name) {
      mkdirSync(SHOTS, { recursive: true });
      let path = `${SHOTS}${name}.png`;
      for (let i = 2; existsSync(path); i++) path = `${SHOTS}${name}-${i}.png`;
      await page.screenshot({ path });
      return path;
    },
    async done() {
      console.log(
        errors.length ? `console errors:\n- ${errors.join('\n- ')}` : 'console errors: none'
      );
      await browser.close();
    },
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [route = '/', name = 'shot', w = '1280', h = '800'] = process.argv.slice(2);
  const app = await open(route, { width: Number(w), height: Number(h) });
  console.log(`screenshot: ${await app.shot(name)}`);
  await app.done();
}
