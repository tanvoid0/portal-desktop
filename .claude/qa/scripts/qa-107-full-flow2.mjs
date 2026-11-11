import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\a32cb83c572c46cb\\3d9c5a03-5565-4268-b94d-cdc9b4983a71\\scratchpad\\qa-agent-test';
function log(...a) { console.log(new Date().toISOString().slice(11, 19), ...a); }

async function waitFor(page, re, timeoutS) {
  for (let i = 0; i < timeoutS; i++) {
    await page.waitForTimeout(1000);
    const text = await page.evaluate(() => document.body.innerText);
    if (re.test(text)) return true;
  }
  return false;
}

const app = await open('agents/run');
const { page } = app;
await page.waitForTimeout(500);

await page.locator('#agent-cwd').fill(CWD);
await page.locator('#agent-prompt').fill('Run the shell command `dir` in the current directory. That is the only thing to do.');
await page.locator('button').filter({ hasText: 'Start run' }).click();

log('waiting for first approval...');
const saw1 = await waitFor(page, /Approval required/i, 60);
log('first approval seen:', saw1);
if (!saw1) { await app.shot('QA-107-flow2-no-first-approval'); await app.done(); process.exit(0); }

const buttons1 = await page.locator('button').allInnerTexts();
log('buttons at first approval:', JSON.stringify(buttons1.filter((b) => /accept|reject|approve/i.test(b))));
await app.shot('QA-107-flow2-first-approval');

await page.locator('button').filter({ hasText: 'Accept once' }).first().click();
log('clicked Accept once');
await page.waitForTimeout(3000);
const afterAccept = await page.evaluate(() => document.body.innerText);
log('ended after accept?', /Run ended/i.test(afterAccept));
await app.shot('QA-107-flow2-after-accept-once');

// Ask for the exact same command again — with no "remember" option shown,
// expect it to ask for approval again every time.
if (!/Run ended/i.test(afterAccept)) {
  const composer = page.locator('textarea');
  await composer.first().fill('Run `dir` again, the exact same command.');
  await page.locator('button').filter({ hasText: 'Send' }).click();
  log('sent follow-up for the same command');
  const saw2 = await waitFor(page, /Approval required/i, 40);
  log('second approval seen for the identical command:', saw2);
  await app.shot('QA-107-flow2-second-approval');
  if (saw2) {
    const buttons2 = await page.locator('button').allInnerTexts();
    log('buttons at second approval:', JSON.stringify(buttons2.filter((b) => /accept|reject|approve/i.test(b))));
    await page.locator('button').filter({ hasText: 'Reject' }).first().click();
    log('clicked Reject');
    await page.waitForTimeout(3000);
    await app.shot('QA-107-flow2-after-reject');
  }
}

// Interrupt (Stop), if a response is actively streaming.
const stopBtn = page.locator('button').filter({ hasText: 'Stop' });
if ((await stopBtn.count()) > 0) {
  await stopBtn.first().click();
  log('clicked Stop (interrupt)');
  await page.waitForTimeout(1500);
  await app.shot('QA-107-flow2-after-interrupt');
}

// Cancel / End run.
const endBtn = page.locator('button').filter({ hasText: 'End run' });
if ((await endBtn.count()) > 0) {
  await endBtn.first().click();
  log('clicked End run');
  await page.waitForTimeout(1500);
  const finalText = await page.evaluate(() => document.body.innerText);
  log('shows ended/new-run state after End run:', /Run ended|New run/i.test(finalText));
  await app.shot('QA-107-flow2-after-end-run');
}

await app.done();
