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
import Module from 'node:module';
import { existsSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// WebView2's CDP implementation omits `browserContextId` on `shared_worker` targets
// (stock Chromium always sets it). Playwright-core asserts on that field and throws
// before the connection ever finishes, which kills every dev-instance that has a Vite
// HMR shared worker alive (i.e. every `tauri:dev` instance). The two lines right after
// the assert already fall back to the default context when it's missing/unknown, so the
// assert is pure sanity-checking we can drop — patched in-memory here, nothing on disk
// changes. Upstream bug, not app code; drop this once Playwright or WebView2 fixes it.
const origCompile = Module.prototype._compile;
Module.prototype._compile = function (content, filename) {
  if (filename.endsWith('coreBundle.js')) {
    content = content.replace(
      'assert(targetInfo.browserContextId, "targetInfo: " + JSON.stringify(targetInfo, null, 2));',
      ''
    );
  }
  return origCompile.call(this, content, filename);
};
const { chromium } = await import('@playwright/test');

const SHOTS = fileURLToPath(new URL('../shots/', import.meta.url));

export async function open(route = '/', { width = 1280, height = 800 } = {}) {
  // CDP_PORT lets a second dev instance (scripts/dev-instance.mjs N, CDP 9222+(N-1))
  // be driven without touching instance 1 on the default :9222.
  const cdpPort = process.env.CDP_PORT ?? '9222';
  let browser, page, mode;
  try {
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`, { timeout: 2000 });
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

  const vitePort = 1420 + (Number(cdpPort) - 9222);
  const base = mode === 'tauri' ? page.url() : `http://localhost:${vitePort}`;
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
      // Shots are kept for demos and docs: never overwrite, add -2, -3, ...
      let path = `${SHOTS}${name}.png`;
      for (let i = 2; existsSync(path); i++) path = `${SHOTS}${name}-${i}.png`;
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
