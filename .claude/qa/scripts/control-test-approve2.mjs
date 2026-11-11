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

const result = await page.evaluate(() => {
  const labels = Array.from(document.querySelectorAll('label[data-label-root]'));
  const card = labels
    .map((l) => l.closest('[data-slot="card-content"]'))
    .find((c) => c && c.textContent.includes("Tanveer's S26 Ultra"));
  if (!card) return { error: 'card not found', bodyHasName: document.body.innerText.includes("S26 Ultra") };

  for (const cap of ['coder', 'terminal']) {
    const lbl = Array.from(card.querySelectorAll('label[data-label-root]')).find(
      (l) => l.textContent.trim() === cap
    );
    const btn = lbl?.querySelector('button[data-slot="switch"]');
    btn?.click();
  }
  const radioLabel = Array.from(card.querySelectorAll('label[data-label-root]')).find(
    (l) => l.textContent.trim() === 'Until revoked'
  );
  radioLabel?.querySelector('button[data-slot="radio-group-item"]')?.click();
  return { preparedCard: true };
});
console.log('prep:', JSON.stringify(result));
await page.waitForTimeout(300);

const clickResult = await page.evaluate(() => {
  const labels = Array.from(document.querySelectorAll('label[data-label-root]'));
  const card = labels
    .map((l) => l.closest('[data-slot="card-content"]'))
    .find((c) => c && c.textContent.includes("Tanveer's S26 Ultra"));
  if (!card) return { error: 'card not found on allow pass' };
  const allowBtn = Array.from(card.querySelectorAll('button')).find((b) => b.textContent.trim() === 'Allow');
  if (!allowBtn) return { error: 'allow button not found' };
  allowBtn.click();
  return { clicked: true };
});
console.log('allow:', JSON.stringify(clickResult));
await page.waitForTimeout(1500);
await page.screenshot({ path: '.claude/qa/shots/control-test-approved.png', fullPage: true });
await browser.close();
