// Minimal CDP driver for the running Tauri app on :9222.
//
// Why this exists: Playwright's `connectOverCDP` (used by app.mjs) throws an
// "Assertion error ... type: shared_worker" against this app's WebView2 — the app keeps a
// SharedWorker attached, and playwright-core 1.64 cannot handle that target type at attach
// time. It is not transient. This driver talks to the single page target's
// webSocketDebuggerUrl directly, so it never sees the worker.
//
//   import { connect } from './cdp.mjs';
//   const app = await connect();
//   await app.nav('/settings/general');        // SvelteKit client router, like a real link
//   await app.clickButton('Refresh');          // asserts tagName === 'BUTTON'
//   await app.shot('ux/UX-003-after-sdk');
//   await app.metrics(1024, 600); ... await app.resetMetrics();
//   app.errors;                                 // console errors + page exceptions
//   await app.close();
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SHOTS = fileURLToPath(new URL('../shots/', import.meta.url));

export async function connect(endpoint = 'http://127.0.0.1:9222') {
  const targets = await fetch(`${endpoint}/json/list`).then((r) => r.json());
  const target = targets.find((t) => t.type === 'page');
  if (!target) throw new Error('no page target on ' + endpoint);

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => {
    ws.onopen = res;
    ws.onerror = rej;
  });

  let id = 0;
  const pending = new Map();
  const errors = [];
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const { res, rej } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? rej(new Error(JSON.stringify(msg.error))) : res(msg.result);
      return;
    }
    if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error')
      errors.push(msg.params.args.map((a) => a.value ?? a.description ?? a.type).join(' '));
    if (msg.method === 'Runtime.exceptionThrown')
      errors.push(
        msg.params.exceptionDetails.exception?.description ?? msg.params.exceptionDetails.text
      );
  };

  const send = (method, params = {}) =>
    new Promise((res, rej) => {
      const myId = ++id;
      pending.set(myId, { res, rej });
      ws.send(JSON.stringify({ id: myId, method, params }));
    });

  await send('Runtime.enable');
  await send('Page.enable');

  const app = {
    errors,
    send,
    /** Evaluate a function in the page; args must be JSON-serializable. */
    async eval(fn, ...args) {
      const expr = `(${fn.toString()})(...${JSON.stringify(args)})`;
      const r = await send('Runtime.evaluate', {
        expression: expr,
        returnByValue: true,
        awaitPromise: true,
      });
      if (r.exceptionDetails)
        throw new Error(
          'page eval threw: ' +
            (r.exceptionDetails.exception?.description ?? r.exceptionDetails.text)
        );
      return r.result.value;
    },
    wait: (ms) => new Promise((r) => setTimeout(r, ms)),
    /** Navigate through the SvelteKit client router (an injected anchor click). */
    async nav(href, settle = 1200) {
      await app.eval((h) => {
        document.getElementById('__navprobe')?.remove();
        const a = document.createElement('a');
        a.href = h;
        a.id = '__navprobe';
        a.textContent = 'go';
        a.style.cssText = 'position:fixed;top:0;left:0;z-index:99999;opacity:0.01';
        document.body.appendChild(a);
        a.click();
      }, href);
      await app.wait(settle);
      return app.eval(() => {
        document.getElementById('__navprobe')?.remove();
        return location.pathname + location.search;
      });
    },
    /**
     * Click a real <button> whose text contains `text`. Returns what it clicked, or null.
     * Guards the role="button" wrapper trap: only elements with tagName === 'BUTTON' match.
     */
    clickButton(text, nth = 0, scope = 'main') {
      return app.eval(
        (t, n, sc) => {
          const root = document.querySelector(sc) ?? document;
          const hits = [...root.querySelectorAll('button')].filter(
            (b) => b.innerText.trim().includes(t) || b.getAttribute('aria-label')?.includes(t)
          );
          const el = hits[n];
          if (!el) return null;
          if (el.tagName !== 'BUTTON') return { error: 'not a BUTTON: ' + el.tagName };
          el.scrollIntoView({ block: 'center' });
          el.click();
          return { clicked: el.innerText.trim() || el.getAttribute('aria-label'), count: hits.length };
        },
        text,
        nth,
        scope
      );
    },
    async metrics(width, height) {
      await send('Emulation.setDeviceMetricsOverride', {
        width,
        height,
        deviceScaleFactor: 1,
        mobile: false,
      });
      await app.wait(600);
    },
    async resetMetrics() {
      await send('Emulation.clearDeviceMetricsOverride');
      await app.wait(600);
    },
    /** Viewport screenshot (the app scrolls inside <main>, so fullPage is meaningless). */
    async shot(name) {
      const dir = SHOTS + name.split('/').slice(0, -1).join('/');
      mkdirSync(dir, { recursive: true });
      let path = `${SHOTS}${name}.png`;
      for (let i = 2; existsSync(path); i++) path = `${SHOTS}${name}-${i}.png`;
      const { data } = await send('Page.captureScreenshot', { format: 'png' });
      writeFileSync(path, Buffer.from(data, 'base64'));
      return path;
    },
    async close() {
      ws.close();
    },
  };
  return app;
}
