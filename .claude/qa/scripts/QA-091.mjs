import { open } from './app.mjs';

const routes = [
  'settings/ai/providers',
  'settings/languages',
  'settings/package-managers',
  'settings/frameworks',
  'settings/ai/training',
  'settings/ides',
  'settings/autonomy',
];
for (const r of routes) {
  const app = await open(r);
  await app.page.waitForTimeout(800);
  const current = await app.page.evaluate(() =>
    [...document.querySelectorAll('[aria-current]')].map(
      (el) => `${el.textContent.trim().slice(0, 30)}:${el.getAttribute('aria-current')}`
    )
  );
  console.log(`${r} aria-current=${JSON.stringify(current)}`);
  await app.done();
}
