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

  const allowBtn = Array.from(card.querySelectorAll('button')).find(
    (b) => b.textContent.trim() === 'Allow'
  );
  return { toggled, hasAllow: !!allowBtn };
});

console.log(JSON.stringify(result, null, 2));
await page.waitForTimeout(300);
await page.screenshot({ path: '.claude/qa/shots/manual-ip-card-caps-selected.png' });

await browser.close();
