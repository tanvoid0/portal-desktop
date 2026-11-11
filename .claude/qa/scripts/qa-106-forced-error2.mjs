// QA-106 re-verify, take 2: intercept the real `http://ipc.localhost/agents_list_backends`
// request Tauri's postMessage fallback makes and force it to fail, then confirm
// the Agents -> Run page shows PageError + retry instead of a silently empty Select.
import { open } from './app.mjs';

const app = await open('agents/run');

await app.page.route('**://ipc.localhost/agents_list_backends', (route) => {
  route.fulfill({ status: 500, body: 'QA-106 simulated failure' });
});

await app.page.reload();
await app.page.waitForLoadState('load');
await app.page.waitForTimeout(2000);

const bodyText = await app.page.evaluate(() => document.body.innerText);
const hasError = /fail|error|try again|retry/i.test(bodyText);
const selectPresent = await app.page.locator('#agent-backend').count();
console.log('page mentions error/retry:', hasError);
console.log('agent-backend select present (expect 0 while erroring):', selectPresent);
await app.shot('QA-106-after-forced-error');

const retryBtn = app.page.locator('button').filter({ hasText: /retry|try again/i });
console.log('retry button count:', await retryBtn.count());
if (await retryBtn.count() > 0) {
  // Clear the route so a retry succeeds, proving the retry path actually re-fetches.
  await app.page.unroute('**://ipc.localhost/agents_list_backends');
  await retryBtn.first().click();
  await app.page.waitForTimeout(1500);
  const after = await app.page.evaluate(() => document.body.innerText);
  const recovered = await app.page.locator('#agent-backend').count();
  console.log('after retry (route cleared), select present:', recovered);
  console.log('after retry, page text sample:', after.slice(0, 300));
  await app.shot('QA-106-after-forced-error-retry-recovered');
}

await app.done();
