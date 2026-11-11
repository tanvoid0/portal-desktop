// QA-106 re-verify: force agents_list_backends to reject and confirm the
// Agents -> Run page now shows PageError + retry instead of a silent empty
// Select. Monkey-patches the Tauri IPC invoke in-page, then reloads.
import { open } from './app.mjs';

const app = await open('agents/run');
await app.page.waitForTimeout(500);

// Patch the Tauri core invoke so agents_list_backends always rejects.
await app.page.addInitScript(() => {
  const w = window;
  const tryPatch = () => {
    const core = w.__TAURI_INTERNALS__;
    if (!core || !core.invoke) return false;
    const orig = core.invoke.bind(core);
    core.invoke = (cmd, ...args) => {
      if (cmd === 'agents_list_backends') {
        return Promise.reject('QA-106 simulated backend failure');
      }
      return orig(cmd, ...args);
    };
    return true;
  };
  if (!tryPatch()) {
    const iv = setInterval(() => { if (tryPatch()) clearInterval(iv); }, 10);
  }
});

await app.page.reload();
await app.page.waitForLoadState('load');
await app.page.waitForTimeout(1500);

const bodyText = await app.page.evaluate(() => document.body.innerText);
const hasError = /fail|error|try again|retry/i.test(bodyText);
const optionCountVisible = await app.page.locator('#agent-backend').count();
console.log('page mentions error/retry:', hasError);
console.log('agent-backend select present (should be false while erroring):', optionCountVisible);
await app.shot('QA-106-after-forced-error');

// Click retry if present, confirm it attempts again (still errors, same state).
const retryBtn = app.page.locator('button').filter({ hasText: /retry|try again/i });
console.log('retry button count:', await retryBtn.count());
if (await retryBtn.count() > 0) {
  await retryBtn.first().click();
  await app.page.waitForTimeout(800);
  const after = await app.page.evaluate(() => document.body.innerText);
  console.log('after retry click, still shows error:', /fail|error|try again|retry/i.test(after));
  await app.shot('QA-106-after-forced-error-retry-clicked');
}

await app.done();
