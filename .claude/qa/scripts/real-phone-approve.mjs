import { chromium } from 'playwright';

const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
const page = browser.contexts()[0].pages().find((p) => p.url().includes('/devices'));
await page.bringToFront();

const result = await page.evaluate(() => {
  const labels = Array.from(document.querySelectorAll('label[data-label-root]'));
  const card = labels
    .map((l) => l.closest('[data-slot="card-content"]'))
    .find((c) => c && c.textContent.includes("Tanveer's S26 Ultra"));
  if (!card) return { error: 'card not found' };

  const toggled = [];
  for (const cap of ['coder', 'terminal', 'agents', 'dashboard', 'projects', 'tasks']) {
    const lbl = Array.from(card.querySelectorAll('label[data-label-root]')).find(
      (l) => l.textContent.trim() === cap
    );
    const btn = lbl?.querySelector('button[data-slot="switch"]');
    if (btn) {
      btn.click();
      toggled.push(cap);
    }
  }

  // "Until revoked" radio
  const radioLabel = Array.from(card.querySelectorAll('label[data-label-root]')).find(
    (l) => l.textContent.trim() === 'Until revoked'
  );
  const radioBtn = radioLabel?.querySelector('button[data-slot="radio-group-item"]');
  radioBtn?.click();

  return { toggled, pickedUntilRevoked: !!radioBtn };
});

console.log('setup:', JSON.stringify(result));
await page.waitForTimeout(300);

const clickResult = await page.evaluate(() => {
  const labels = Array.from(document.querySelectorAll('label[data-label-root]'));
  const card = labels
    .map((l) => l.closest('[data-slot="card-content"]'))
    .find((c) => c && c.textContent.includes("Tanveer's S26 Ultra"));
  if (!card) return { error: 'card not found on second pass' };
  const allowBtn = Array.from(card.querySelectorAll('button')).find(
    (b) => b.textContent.trim() === 'Allow'
  );
  if (!allowBtn) return { error: 'allow button not found' };
  allowBtn.click();
  return { clicked: true };
});

console.log('allow:', JSON.stringify(clickResult));
await page.waitForTimeout(1500);
await page.screenshot({ path: '.claude/qa/screenshots/real-phone-approved.png', fullPage: true });

await browser.close();
