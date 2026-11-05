// QA helper. Drives the real Tauri app over CDP (:9222) when it is up, else the
// vite dev server (:1420) in Chromium — browser mode has no backend, invoke returns empties.
//
//   node .claude/qa/scripts/app.mjs projects/audit [name]   → screenshot + console errors
//   (no leading slash on the CLI — Git Bash mangles it; `''` is the home page)
//
// Repro scripts import it:
//   import { open } from './app.mjs';
//   const app = await open('projects');
//   await app.page.getByRole('button', { name: 'Add' }).click();
//   await app.shot('QA-007-after-add');
//   await app.done();
import { chromium } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SHOTS = fileURLToPath(new URL('../shots/', import.meta.url));

export async function open(route = '/', { width = 1280, height = 800 } = {}) {
  let browser, page, mode;
  try {
    browser = await chromium.connectOverCDP('http://127.0.0.1:9222', { timeout: 2000 });
    page = browser.contexts()[0].pages()[0];
    mode = 'tauri';
  } catch {
    browser = await chromium.launch({ channel: 'msedge' }); // system Edge = WebView2's engine, no download
    page = await browser.newPage({ viewport: { width, height } });
    mode = 'browser (no backend: invoke returns empty values)';
  }
  const errors = [];
  page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
  page.on('pageerror', (e) => errors.push(e.message));

  const base = mode === 'tauri' ? page.url() : 'http://localhost:1420';
  // Routes are passed without the leading slash: Git Bash rewrites `/projects` into a Windows path.
  await page.goto(new URL(route.replace(/^\/*/, '/'), base).href);
  await page.waitForLoadState('load');
  await page.waitForTimeout(1000); // let queries settle

  return {
    page,
    mode,
    errors,
    async shot(name) {
      mkdirSync(SHOTS, { recursive: true });
      const path = `${SHOTS}${name}.png`;
      await page.screenshot({ path, fullPage: true });
      return path;
    },
    // Disconnects; does not close the real app's window.
    async done() {
      console.log(`mode: ${mode}`);
      console.log(
        errors.length ? `console errors:\n- ${errors.join('\n- ')}` : 'console errors: none'
      );
      await browser.close();
    },
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [route = '/', name = 'shot'] = process.argv.slice(2);
  const app = await open(route);
  console.log(`screenshot: ${await app.shot(name)}`);
  await app.done();
}
