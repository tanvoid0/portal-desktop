// QA-107 full flow: approval banner wording, Accept once / Accept & remember
// / Reject, interrupt, cancel/end run, and whether "remember" actually
// suppresses a repeat of the same tool call. One continuous page session
// (no reload) so we don't drop the live run.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\a32cb83c572c46cb\\3d9c5a03-5565-4268-b94d-cdc9b4983a71\\scratchpad\\qa-agent-test';

function log(...a) { console.log(new Date().toISOString().slice(11, 19), ...a); }

async function waitForApproval(page, timeoutS = 60) {
  for (let i = 0; i < timeoutS; i++) {
    await page.waitForTimeout(1000);
    const count = await page.locator('text=Approval required').count();
    if (count > 0) return true;
    const text = await page.evaluate(() => document.body.innerText);
    if (/Run ended/i.test(text)) return false;
  }
  return false;
}

const app = await open('agents/run');
const { page } = app;
await page.waitForTimeout(500);

await page.locator('#agent-cwd').fill(CWD);
await page.locator('#agent-prompt').fill(
  'Run the shell command `dir` in the current directory. That is the only thing to do.'
);
await page.locator('button').filter({ hasText: 'Start run' }).click();

log('waiting for first approval...');
const sawFirst = await waitForApproval(page, 60);
log('first approval seen:', sawFirst);
await app.shot('QA-107-flow-1-first-approval');

if (sawFirst) {
  const buttons = await page.locator('button').allInnerTexts();
  const relevant = buttons.filter((b) => /accept|reject|approve/i.test(b));
  log('approval buttons found:', JSON.stringify(relevant));

  // Exercise "Accept & remember" with the default suggested rule, unchanged.
  const rememberBtn = page.locator('button').filter({ hasText: 'Accept & remember' });
  log('Accept & remember button count:', await rememberBtn.count());
  await rememberBtn.first().click();
  log('clicked Accept & remember');

  await page.waitForTimeout(3000);
  const afterFirst = await page.evaluate(() => document.body.innerText);
  log('ended after first decision?', /Run ended/i.test(afterFirst));
  await app.shot('QA-107-flow-2-after-remember-click');

  // If the run ended (single dir call, no more approvals needed), send a
  // follow-up asking for the exact same command again to test whether
  // "remember" suppressed the second approval.
  if (/Run ended/i.test(afterFirst)) {
    log('run ended after one command; starting a NEW run to re-test the same tool for remember.');
  } else {
    // Still running / composer available — ask for the same command again.
    const composer = page.locator('textarea');
    if ((await composer.count()) > 0 && !(await composer.first().isDisabled())) {
      await composer.first().fill('Run `dir` again, exactly the same command.');
      await page.locator('button').filter({ hasText: 'Send' }).click();
      log('sent follow-up asking for the same command again');
      const sawSecond = await waitForApproval(page, 40);
      log('second approval seen for the SAME command (expect false if remember worked):', sawSecond);
      await app.shot('QA-107-flow-3-second-same-command');
      if (sawSecond) {
        // Reject it this time, to exercise Reject's wording/behavior too.
        const rejectBtn = page.locator('button').filter({ hasText: 'Reject' });
        await rejectBtn.first().click();
        log('clicked Reject');
        await page.waitForTimeout(2000);
        await app.shot('QA-107-flow-4-after-reject');
      }
    }
  }
}

// Interrupt test: if still running (streaming), hit Stop.
const runningText = await page.evaluate(() => document.body.innerText);
const stopBtn = page.locator('button').filter({ hasText: 'Stop' });
if ((await stopBtn.count()) > 0) {
  await stopBtn.first().click();
  log('clicked Stop (interrupt)');
  await page.waitForTimeout(1500);
  await app.shot('QA-107-flow-5-after-interrupt');
}

// Cancel/End run test.
const endBtn = page.locator('button').filter({ hasText: 'End run' });
if ((await endBtn.count()) > 0) {
  await endBtn.first().click();
  log('clicked End run');
  await page.waitForTimeout(1500);
  const finalText = await page.evaluate(() => document.body.innerText);
  log('shows "Run ended" / "New run" after End run:', /Run ended|New run/i.test(finalText));
  await app.shot('QA-107-flow-6-after-end-run');
}

await app.done();
