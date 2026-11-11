import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\a32cb83c572c46cb\\3d9c5a03-5565-4268-b94d-cdc9b4983a71\\scratchpad\\qa-agent-test';
function log(...a) { console.log(new Date().toISOString().slice(11, 19), ...a); }

const app = await open('agents/run');
const { page } = app;
await page.waitForTimeout(500);

await page.locator('#agent-cwd').fill(CWD);
await page.locator('#agent-prompt').fill(
  'Run the shell command `dir`, then write a two-paragraph summary of what a README file is for. Take your time on the summary.'
);
await page.locator('button').filter({ hasText: 'Start run' }).click();

// Wait for and accept the first approval.
for (let i = 0; i < 60; i++) {
  await page.waitForTimeout(1000);
  if ((await page.locator('text=Approval required').count()) > 0) break;
}
const sawApproval = (await page.locator('text=Approval required').count()) > 0;
log('approval seen:', sawApproval);
if (sawApproval) {
  await page.locator('button').filter({ hasText: 'Accept once' }).first().click();
  log('accepted');
}

// While it's streaming the summary, hit Stop (interrupt).
let interrupted = false;
for (let i = 0; i < 20; i++) {
  await page.waitForTimeout(1000);
  const stopBtn = page.locator('button').filter({ hasText: 'Stop' });
  if ((await stopBtn.count()) > 0) {
    await stopBtn.first().click();
    log('clicked Stop at t+', i, 's');
    interrupted = true;
    break;
  }
}
log('interrupt attempted:', interrupted);
await page.waitForTimeout(1500);
await app.shot('QA-107-interrupt-result');

const midText = await page.evaluate(() => document.body.innerText);
log('running indicator gone after Stop (expect true):', !/Thinking…|Thinking\.\.\./.test(midText));

// Now End run.
const endBtn = page.locator('button').filter({ hasText: 'End run' });
if ((await endBtn.count()) > 0) {
  await endBtn.first().click();
  log('clicked End run');
  await page.waitForTimeout(1500);
  const finalText = await page.evaluate(() => document.body.innerText);
  log('ended state shown:', /Run ended/i.test(finalText), '| New run button:', /New run/i.test(finalText));
  await app.shot('QA-107-end-run-result');
}

await app.done();
