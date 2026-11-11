import Module from 'node:module';
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

const browser = await chromium.connectOverCDP('http://127.0.0.1:9224', { timeout: 5000 });
const page = browser.contexts()[0].pages()[0];
const logs = [];
page.on('console', (m) => {
  if (m.type() === 'error') logs.push(m.text());
});
const base = page.url();
await page.goto(new URL('/sdk', base).href);
await page.waitForTimeout(8000);
for (const l of logs.filter((l) => l.startsWith('QA097'))) console.log(l);
console.log('---page text snippet---');
const text = await page.evaluate(() => document.body.innerText);
console.log(text.includes('Loading SDK data') ? 'STILL STUCK' : 'RESOLVED');
await browser.close();
