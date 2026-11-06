// Repro: /github says Connected, but every data page fails with a decrypt error and offers only "Retry".
import { open } from './app.mjs';
const app = await open('github');
const p = app.page;
await p.waitForTimeout(2000);
console.log('home shows Connected:', await p.getByText('Connected', { exact: true }).count());
for (const r of ['github/repos', 'github/issues']) {
  await p.goto(new URL('/' + r, p.url()).href);
  await p.waitForTimeout(5000);
  console.log(r, '->', (await p.locator('main').innerText()).replace(/\s+/g, ' ').slice(-120));
}
await app.shot('QA-045-before-github-decrypt-error-issues');
await app.done();
