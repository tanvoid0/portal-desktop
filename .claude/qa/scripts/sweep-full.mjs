import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const routes = (process.argv[2] || 'automation,database,deployments,documents,logs,pipeline-runs,scripts,system,utilities,blocks,connect,terminal,tasks,projects').split(',');
const theme = process.argv[3] || 'light';
const [w, h] = (process.argv[4] || '1280x800').split('x').map(Number);
const tag = process.argv[5] || '';
const app = await open('', {});
const p = app.page;
try { await p.setViewportSize({ width: w, height: h }); } catch (e) { console.log('viewport fail', e.message); }
await p.evaluate((t) => localStorage.setItem('theme', t), theme);
console.log('inner', await p.evaluate(() => [innerWidth, innerHeight]));
for (const r of routes) {
  app.errors.length = 0;
  await p.goto(new URL('/' + r, p.url()).href);
  await p.waitForLoadState('load');
  await p.waitForTimeout(1500);
  await dismissVault(p, 1500);
  const info = await p.evaluate(() => ({
    h1: document.querySelector('h1')?.textContent?.trim(),
    hscroll: document.documentElement.scrollWidth > innerWidth || document.body.scrollWidth > innerWidth,
    unl: [...document.querySelectorAll('button,a[href]')].filter(b => !(b.textContent||'').trim() && !b.getAttribute('aria-label') && !b.getAttribute('title')).map(b=>b.outerHTML.slice(0,120)),
    unlabeled: [...document.querySelectorAll('button,a[href]')].filter(b => !(b.textContent||'').trim() && !b.getAttribute('aria-label') && !b.getAttribute('title')).length,
    dark: document.documentElement.classList.contains('dark'),
  }));
  const path = await app.shot(`sweep-${r}-${theme}${tag}`);
  console.log(r, JSON.stringify(info), app.errors.slice(0,4).join(' | '));
}
await p.evaluate(() => localStorage.removeItem('theme'));
await app.done();
