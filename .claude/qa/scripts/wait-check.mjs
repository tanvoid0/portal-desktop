// Does a route finish loading? Samples main text + spinner count over 20s.
import { chromium } from '@playwright/test';

const route = process.argv[2] ?? '';
const browser = await chromium.connectOverCDP('http://127.0.0.1:9222', { timeout: 5000 });
const page = browser.contexts()[0].pages()[0];
const errors = [];
page.on('console', (m) => m.type() === 'error' && errors.push(m.text().slice(0, 200)));
page.on('pageerror', (e) => errors.push('pageerror: ' + e.message.slice(0, 200)));
await page.goto('http://localhost:1420/' + route.replace(/^\/*/, ''));
for (const t of [1, 3, 6, 10, 15, 20]) {
  await page.waitForTimeout(t * 1000 - (t === 1 ? 0 : 0) - 0);
  const s = await page.evaluate(() => ({
    spin: document.querySelectorAll('.animate-spin,[data-loading="true"]').length,
    txt: ((document.querySelector('main') || document.body).innerText || '')
      .replace(/\s+/g, ' ')
      .trim()
      .slice(0, 180),
    title: document.title,
  }));
  console.log(JSON.stringify({ at: t, ...s }));
  if (s.spin === 0 && !/loading|connecting/i.test(s.txt)) break;
}
console.log('errors:', JSON.stringify(errors));
await browser.close();
