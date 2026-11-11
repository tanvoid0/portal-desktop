// Re-verify QA-107 (axum :token panic fix) live on instance-2 (CDP :9223).
// Two runs against the real "claude-code" backend:
//   Run 1: shell command -> terminal tool-call approval -> Accept once -> Stop (interrupt) -> End run (cancel)
//   Run 2: file write -> coder tool-call approval -> Reject -> End run (cancel)
// Prints clear PASS/FAIL lines for each step instead of just screenshotting.
import { open } from './app.mjs';

const CWD =
  'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\62c7a53de9098bc2\\67f9783e-7f7f-4003-ad45-3b6622aaaa6f\\scratchpad\\qa-agent-test';

async function waitFor(page, predicate, timeoutMs, label) {
  const steps = Math.ceil(timeoutMs / 1000);
  for (let i = 0; i < steps; i++) {
    if (await predicate()) return true;
    await page.waitForTimeout(1000);
  }
  console.log(`TIMEOUT waiting for: ${label}`);
  return false;
}

const app = await open('agents/run');
console.log('mode:', app.mode);
await app.page.waitForTimeout(500);

// ---------- Run 1: terminal tool call ----------
await app.page.locator('#agent-cwd').fill(CWD);
await app.page
  .locator('#agent-prompt')
  .fill('Run the shell command `dir` in the current directory to list files, then stop.');
await app.shot('QA-107-after-run1-filled');

const startBtn = app.page.locator('button').filter({ hasText: 'Start run' });
await startBtn.click();

const gotPastStarting = await waitFor(
  app.page,
  async () => !(await startBtn.isVisible().catch(() => false)),
  15000,
  'Start run button to disappear (run started)'
);
console.log('RESULT run1 left "Starting..." state:', gotPastStarting);

const sawApproval1 = await waitFor(
  app.page,
  async () => /Approval required/i.test(await app.page.evaluate(() => document.body.innerText)),
  30000,
  'approval banner (run1)'
);
console.log('RESULT run1 saw approval banner:', sawApproval1);
await app.shot('QA-107-after-run1-approval-or-timeout');

if (sawApproval1) {
  const acceptBtn = app.page.locator('button').filter({ hasText: 'Accept once' }).first();
  await acceptBtn.click();
  console.log('clicked Accept once (run1)');

  const sawRunning = await waitFor(
    app.page,
    async () => await app.page.locator('button').filter({ hasText: 'Stop' }).first().isVisible().catch(() => false),
    15000,
    'Stop button (run running after accept)'
  );
  console.log('RESULT run1 running after accept (Stop visible):', sawRunning);

  if (sawRunning) {
    await app.page.locator('button').filter({ hasText: 'Stop' }).first().click();
    console.log('clicked Stop (interrupt) on run1');
    await app.page.waitForTimeout(2000);
    await app.shot('QA-107-after-run1-interrupt');
  }
}

const endRunBtn1 = app.page.locator('button').filter({ hasText: 'End run' }).first();
if (await endRunBtn1.isVisible().catch(() => false)) {
  await endRunBtn1.click();
  console.log('clicked End run (cancel) on run1');
  await app.page.waitForTimeout(1500);
}
await app.shot('QA-107-after-run1-ended');

const run1Errors = [...app.errors];
app.errors.length = 0;

// ---------- Run 2: new run, coder (file write) tool call ----------
const newRunBtn = app.page.locator('button').filter({ hasText: 'New run' }).first();
if (await newRunBtn.isVisible().catch(() => false)) {
  await newRunBtn.click();
  await app.page.waitForTimeout(500);
}

await app.page.locator('#agent-cwd').fill(CWD);
await app.page
  .locator('#agent-prompt')
  .fill('Create a file named qa-107-probe.txt with the text "hello" in it, then stop.');
await app.shot('QA-107-after-run2-filled');

const startBtn2 = app.page.locator('button').filter({ hasText: 'Start run' });
await startBtn2.click();

const gotPastStarting2 = await waitFor(
  app.page,
  async () => !(await startBtn2.isVisible().catch(() => false)),
  15000,
  'Start run button to disappear (run2 started)'
);
console.log('RESULT run2 left "Starting..." state:', gotPastStarting2);

const sawApproval2 = await waitFor(
  app.page,
  async () => /Approval required/i.test(await app.page.evaluate(() => document.body.innerText)),
  30000,
  'approval banner (run2)'
);
console.log('RESULT run2 saw approval banner:', sawApproval2);
await app.shot('QA-107-after-run2-approval-or-timeout');

if (sawApproval2) {
  const rejectBtn = app.page.locator('button').filter({ hasText: 'Reject' }).first();
  await rejectBtn.click();
  console.log('clicked Reject (run2)');
  await app.page.waitForTimeout(2000);
  await app.shot('QA-107-after-run2-reject');
}

const endRunBtn2 = app.page.locator('button').filter({ hasText: 'End run' }).first();
if (await endRunBtn2.isVisible().catch(() => false)) {
  await endRunBtn2.click();
  console.log('clicked End run (cancel) on run2');
  await app.page.waitForTimeout(1500);
}
await app.shot('QA-107-after-run2-ended');

console.log('--- console errors seen during run1 ---');
console.log(run1Errors.length ? run1Errors.join('\n') : '(none)');

await app.done();
