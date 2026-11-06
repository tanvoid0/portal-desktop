// Whole-app route sweep against the running Tauri app over CDP.
import { chromium } from '@playwright/test';
import { mkdirSync, existsSync } from 'node:fs';

const SHOTS = 'D:/projects/production/portal/portal_desktop/.claude/qa/shots/';
const ROUTES = process.argv.slice(2);

const browser = await chromium.connectOverCDP('http://127.0.0.1:9222', { timeout: 5000 });
const page = browser.contexts()[0].pages()[0];
mkdirSync(SHOTS, { recursive: true });

for (const route of ROUTES) {
  const errors = [];
  const onC = (m) => m.type() === 'error' && errors.push(m.text().slice(0, 300));
  const onE = (e) => errors.push('pageerror: ' + e.message.slice(0, 300));
  page.on('console', onC);
  page.on('pageerror', onE);
  const t0 = Date.now();
  try {
    await page.goto('http://localhost:1420/' + route.replace(/^\/*/, ''));
    await page.waitForLoadState('load');
    await page.waitForTimeout(2500);
  } catch (e) {
    errors.push('nav failed: ' + e.message.slice(0, 200));
  }
  const info = await page.evaluate(() => {
    const t = (el) => (el?.innerText || '').replace(/\s+/g, ' ').trim().slice(0, 160);
    const vis = (el) => el.offsetParent !== null || el.getClientRects().length > 0;
    const main = document.querySelector('main') || document.body;
    return {
      title: document.title,
      h1: t(document.querySelector('h1')),
      mainText: t(main).slice(0, 400),
      buttons: [...document.querySelectorAll('button')].filter(vis).length,
      inputs: [...document.querySelectorAll('input,textarea,select')].filter(vis).length,
      links: [...document.querySelectorAll('a')].filter(vis).length,
      spinners: document.querySelectorAll('.animate-spin,[data-loading="true"]').length,
      skeletons: document.querySelectorAll('[class*="skeleton"],[data-skeleton]').length,
      // obvious failure/empty copy
      bad: [...document.querySelectorAll('*')]
        .filter(
          (el) =>
            el.children.length === 0 &&
            vis(el) &&
            /failed|error|undefined|NaN|null|unavailable|went wrong/i.test(el.textContent || '')
        )
        .slice(0, 6)
        .map((el) => (el.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 120)),
      imgsBroken: [...document.images].filter((i) => i.complete && i.naturalWidth === 0).length,
      bodyScrollW: document.documentElement.scrollWidth,
      viewportW: window.innerWidth,
    };
  });
  const name = 'sweep-' + (route === '' ? 'home' : route.replace(/\//g, '-'));
  let path = SHOTS + name + '.png';
  for (let i = 2; existsSync(path); i++) path = `${SHOTS}${name}-${i}.png`;
  await page.screenshot({ path });
  page.off('console', onC);
  page.off('pageerror', onE);
  console.log(
    JSON.stringify({ route, ms: Date.now() - t0, shot: path.split('/').pop(), ...info, errors })
  );
}
await browser.close();
