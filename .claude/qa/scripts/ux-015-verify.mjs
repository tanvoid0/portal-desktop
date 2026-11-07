// UX-015 / UX-005 verification: settings routes collapsed, nav group labels, no double heading.
// In-app navigation goes through the SvelteKit client router (an injected anchor click), the
// way a real link or bookmark resolves inside the webview; a raw page.goto reboots the app.
import { open } from './app.mjs';

const app = await open('settings/general');
const { page } = app;

async function navigate(href) {
  await page.evaluate((h) => {
    document.getElementById('__navprobe')?.remove();
    const a = document.createElement('a');
    a.href = h;
    a.id = '__navprobe';
    a.textContent = 'go';
    a.style.cssText = 'position:fixed;top:0;left:0;z-index:99999;background:#f00;color:#fff';
    document.body.appendChild(a);
  }, href);
  await page.locator('#__navprobe').click();
  await page.waitForTimeout(700);
  const info = await page.evaluate(() => {
    document.getElementById('__navprobe')?.remove();
    const cards = [...document.querySelectorAll('[data-slot="card"]')].filter(
      (e) => e.getBoundingClientRect().x > 400
    );
    return {
      path: location.pathname + location.search,
      notFound: /\b404\b|Not found/i.test(document.body.innerText),
      contentCards: cards.length,
      firstCardY: cards[0] ? Math.round(cards[0].getBoundingClientRect().y) : null,
      headings: [...document.querySelectorAll('h1,h2')].map((e) => e.innerText.trim()).filter(Boolean),
    };
  });
  console.log(`${href.padEnd(44)} -> ${info.path.padEnd(30)} 404:${info.notFound} cards:${info.contentCards} firstY:${info.firstCardY} h:${JSON.stringify(info.headings.slice(2))}`);
  return info;
}

console.log('--- live routes');
for (const r of [
  '/settings/general',
  '/settings/updates',
  '/settings/editor',
  '/settings/terminal',
  '/settings/ides',
  '/settings/languages',
  '/settings/package-managers',
  '/settings/frameworks',
  '/settings/github',
  '/settings/theme',
  '/settings/learning',
  '/settings/ai/providers',
  '/settings/ai/training',
  '/settings/autonomy',
])
  await navigate(r);

console.log('--- retired paths (must redirect, not 404)');
for (const r of [
  '/settings/frameworks-languages',
  '/settings/frameworks-languages?tab=languages',
  '/settings/frameworks-languages?tab=package-managers',
  '/settings/frameworks-languages?tab=frameworks',
  '/settings/ai',
  '/settings',
  '/settings/framework-ides',
  '/ai/providers',
])
  await navigate(r);

console.log('--- nav walk');
await navigate('/settings/general');
const groups = await page.evaluate(() =>
  [...document.querySelectorAll('nav.space-y-2 div.uppercase')].map((d) => d.innerText.trim())
);
console.log('group labels (no path, not clickable):', JSON.stringify(groups));
const labels = await page.evaluate(() =>
  [...document.querySelectorAll('nav.space-y-2 button')].map((b) => b.innerText.split('\n')[0].trim())
);
for (const label of labels) {
  await page.locator('nav.space-y-2 button').filter({ hasText: label }).first().click();
  await page.waitForTimeout(500);
  const info = await page.evaluate(() => ({
    path: location.pathname,
    notFound: /\b404\b|Not found/i.test(document.body.innerText),
    cards: [...document.querySelectorAll('[data-slot="card"]')].filter((e) => e.getBoundingClientRect().x > 400).length,
  }));
  console.log(`click ${label.padEnd(22)} -> ${info.path.padEnd(28)} 404:${info.notFound} cards:${info.cards}`);
}

await navigate('/settings/general');
console.log('shot:', await app.shot('ux/ux-015-settings-general-after'));
await navigate('/settings/ai/training');
console.log('shot:', await app.shot('ux/ux-015-settings-ai-training-after'));
await app.done();
