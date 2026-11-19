// UX review shots: coder (portal engine) + agents backends.
import { open } from './app.mjs';

const app = await open('ai/coder');
const { page } = app;
const mode = process.argv[2] ?? 'probe';

if (mode === 'probe') {
  // Toggle the "All" filter to only workspaces with sessions, then dump sidebar text.
  await page.locator('button').filter({ hasText: /^All$/ }).first().click().catch(() => {});
  await page.waitForTimeout(800);
  const side = await page.evaluate(() => {
    const h = [...document.querySelectorAll('*')].find((e) => e.textContent?.trim() === 'Sessions');
    let el = h;
    for (let i = 0; i < 6 && el; i++) el = el.parentElement;
    return el?.innerText.slice(0, 1500);
  });
  console.log(side);
  const tabTexts = await page.evaluate(() =>
    [...document.querySelectorAll('button, a, [role="tab"]')]
      .filter((e) => /^(Portal|Claude Code|Codex|Cursor)$/.test(e.textContent?.trim() ?? ''))
      .map((e) => `${e.tagName}:${e.textContent.trim()}:${e.getAttribute('href') ?? ''}`)
  );
  console.log('tabs', tabTexts);
  await app.shot('ux-coder-portal-sessions');
} else if (mode === 'open') {
  // node ux-coder-review.mjs open "<row text prefix>" <name>
  await page.locator('button').filter({ hasText: /^All$/ }).first().click().catch(() => {});
  await page.waitForTimeout(600);
  await page.getByText(process.argv[3], { exact: false }).first().click();
  await page.waitForTimeout(2500);
  await app.shot(`ux-coder-${process.argv[4]}`);
  const btns = page.locator('[data-slot="scroll-area-viewport"] button');
  const n = await btns.count();
  for (let j = 0; j < Math.min(n, 14); j++) await btns.nth(j).click({ timeout: 800 }).catch(() => {});
  await page.waitForTimeout(600);
  await app.shot(`ux-coder-${process.argv[4]}-expanded`);
} else if (mode === 'csess') {
  await page.getByText('Claude Code', { exact: true }).first().click();
  await page.waitForTimeout(1500);
  await page.getByText(process.argv[3]).first().click();
  await page.waitForTimeout(3000);
  await app.shot('ux-coder-claude-session');
  const btns = page.locator('main button');
  const n = await btns.count();
  for (let j = 0; j < Math.min(n, 30); j++) { const t = await btns.nth(j).innerText().catch(()=>''); if (/Read|Ran|Edited|Wrote|Searched|Listed|Bash|Todo|Task/.test(t)) await btns.nth(j).click({timeout:800}).catch(()=>{}); }
  await page.waitForTimeout(600);
  await app.shot('ux-coder-claude-session-expanded');
} else if (mode === 'agents') {
  await page.getByText(process.argv[3], { exact: true }).first().click();
  await page.waitForTimeout(2500);
  const side = await page.evaluate(() => document.body.innerText.slice(0, 1200));
  console.log(side);
  await app.shot(`ux-coder-${process.argv[4]}`);
}
await app.done();
// claude-session mode appended: node ux-coder-review.mjs csess "<row prefix>"
