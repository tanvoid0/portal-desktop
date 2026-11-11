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

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages().find((p) => p.url().includes('/devices'));
await page.bringToFront();
await page.getByRole('button', { name: 'Refresh' }).click().catch(() => {});
await page.waitForTimeout(1000);
await page.screenshot({ path: '.claude/qa/shots/control-test-pending-scrolled.png', fullPage: true });
console.log(await page.evaluate(() => document.body.innerText));
await browser.close();
