// UX-015 verifier over raw CDP (app.mjs/Playwright cannot attach to this app — see cdp.mjs).
// Walks every settings nav entry, every retired path, the group labels and the collapsed rail.
//   node .claude/qa/scripts/ux-settings-nav.mjs
import { connect } from './cdp.mjs';

const LIVE = [
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
];
const RETIRED = [
  '/settings',
  '/settings/frameworks-languages',
  '/settings/frameworks-languages?tab=languages',
  '/settings/frameworks-languages?tab=package-managers',
  '/settings/frameworks-languages?tab=frameworks',
  '/settings/ai',
  '/settings/framework-ides',
];

const app = await connect();

const inspect = () =>
  app.eval(() => {
    const main = document.querySelector('main');
    const panel = main.querySelector('[data-slot="card"], .space-y-6') ?? main;
    return {
      path: location.pathname + location.search,
      notFound: /\b404\b|Not found/i.test(main.innerText),
      h1: [...main.querySelectorAll('h1')].map((e) => e.innerText.trim()),
      h2: [...main.querySelectorAll('h2')].map((e) => e.innerText.trim()),
      activeNav: [...document.querySelectorAll('nav.space-y-2 button')]
        .filter((b) => b.className.includes('bg-accent'))
        .map((b) => b.innerText.split('\n')[0].trim()),
      firstLine: panel.innerText.split('\n').filter(Boolean).slice(0, 3).join(' / '),
    };
  });

console.log('--- live settings routes');
for (const r of LIVE) {
  await app.nav(r, 1200);
  const i = await inspect();
  console.log(
    `${r.padEnd(30)} -> ${i.path.padEnd(28)} 404:${i.notFound} active:${JSON.stringify(i.activeNav)} h1:${JSON.stringify(i.h1)} h2:${JSON.stringify(i.h2.slice(0, 4))}`
  );
}

console.log('--- retired paths (must redirect, not 404)');
for (const r of RETIRED) {
  await app.nav(r, 1500);
  const i = await inspect();
  console.log(`${r.padEnd(46)} -> ${i.path.padEnd(28)} 404:${i.notFound}`);
}

console.log('--- group labels must not be buttons/links');
await app.nav('/settings/general', 1200);
console.log(
  JSON.stringify(
    await app.eval(() => {
      const nav = document.querySelector('nav.space-y-2');
      const out = [];
      for (const label of ['Frameworks & Languages', 'AI']) {
        const el = [...nav.querySelectorAll('*')].find(
          (e) => e.children.length === 0 && e.textContent.trim() === label
        );
        const clickable = el?.closest('button,a,[role="button"],[onclick]');
        out.push({ label, found: !!el, insideClickable: clickable ? clickable.tagName : null });
      }
      return out;
    }),
    null,
    1
  )
);

console.log('--- nav button walk (click every entry)');
const labels = await app.eval(() =>
  [...document.querySelectorAll('nav.space-y-2 button')].map((b) =>
    b.innerText.split('\n')[0].trim()
  )
);
for (let n = 0; n < labels.length; n++) {
  const res = await app.eval((i) => {
    const b = [...document.querySelectorAll('nav.space-y-2 button')][i];
    if (!b) return null;
    b.click();
    return b.innerText.split('\n')[0].trim();
  }, n);
  await app.wait(900);
  const i = await inspect();
  console.log(
    `click ${String(res).padEnd(22)} -> ${i.path.padEnd(28)} 404:${i.notFound} active:${JSON.stringify(i.activeNav)}`
  );
}

console.log('errors:', JSON.stringify(app.errors.slice(0, 10)));
await app.close();
