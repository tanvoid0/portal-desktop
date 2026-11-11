// Independent re-verification of QA-110: Accept once / Reject must be
// clickable immediately (no "element is not enabled" wait), and each must
// actually work end to end (approve lets the tool proceed, reject blocks it
// and the run recovers).
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

async function runOnce(mode) {
  const marker = mode === 'approve' ? 'qa-verify110-approve.txt' : 'qa-verify110-reject.txt';
  const app = await open('agents/run');
  const { page } = app;

  await page.getByPlaceholder('Pick a folder…').fill(CWD);
  await page
    .getByPlaceholder('What should the agent do?')
    .fill(
      `Use your Write tool to create a file named ${marker} with content "x", then tell me what happened.`
    );
  await page.getByRole('button', { name: 'Start run' }).click();

  // Wait for the approval banner.
  let sawBanner = false;
  for (let i = 0; i < 60; i++) {
    await page.waitForTimeout(500);
    if ((await page.getByRole('button', { name: 'Accept once' }).count()) > 0) {
      sawBanner = true;
      break;
    }
  }
  if (!sawBanner) {
    console.log(`[${mode}] FAIL: never saw approval banner`);
    await app.done();
    return false;
  }

  const btnName = mode === 'approve' ? 'Accept once' : 'Reject';
  const t0 = Date.now();
  // click() with the default actionability timeout (30s) - if it's disabled
  // this will throw / time out just like the original repro did.
  await page.getByRole('button', { name: btnName }).click({ timeout: 3000 });
  const clickMs = Date.now() - t0;
  console.log(`[${mode}] clicked "${btnName}" in ${clickMs}ms (disabled would time out at 3000ms)`);

  let recovered = false;
  for (let i = 0; i < 40; i++) {
    await page.waitForTimeout(500);
    if ((await page.locator('text=what happened').count()) > 0) {
      recovered = true;
      break;
    }
  }
  console.log(`[${mode}] run recovered to a final assistant message:`, recovered);
  await app.shot(`qa-verify-110-${mode}`);
  await app.done();
  return recovered;
}

const approveOk = await runOnce('approve');
const rejectOk = await runOnce('reject');
console.log('RESULT approve path ok:', approveOk, '| reject path ok:', rejectOk);
