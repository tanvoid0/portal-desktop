// Sweep: Agents -> Run end-to-end. Start a Claude Code run in a throwaway
// scratch dir, trigger a tool-approval banner, exercise Accept once / Accept
// & remember / Reject, interrupt, cancel, and check "Accept & remember"
// actually suppresses the next identical approval.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\a32cb83c572c46cb\\3d9c5a03-5565-4268-b94d-cdc9b4983a71\\scratchpad\\qa-agent-test';

const app = await open('agents/run');
await app.page.waitForTimeout(500);

await app.page.locator('#agent-cwd').fill(CWD);
await app.page.locator('#agent-prompt').fill(
  'Run the shell command `dir` in the current directory to list files. Then run `dir` again.'
);
await app.shot('qa-sweep-agents-run1-filled');

const startBtn = app.page.locator('button').filter({ hasText: 'Start run' });
await startBtn.click();

// Wait for the approval banner (shared coder ApprovalBanner: "Approval required").
let sawApproval = false;
for (let i = 0; i < 30; i++) {
  await app.page.waitForTimeout(1000);
  const text = await app.page.evaluate(() => document.body.innerText);
  if (/Approval required/i.test(text)) { sawApproval = true; break; }
  if (/Run ended/i.test(text) && i > 3) break;
}
console.log('saw approval banner:', sawApproval);
await app.shot('qa-sweep-agents-run1-approval-or-timeout');

if (sawApproval) {
  const bodyText = await app.page.evaluate(() => document.body.innerText);
  console.log('--- page text around approval ---');
  console.log(bodyText.slice(bodyText.indexOf('Approval required') - 200, bodyText.indexOf('Approval required') + 400));

  // Click "Accept & remember" for the first approval.
  const rememberBtn = app.page.locator('button').filter({ hasText: 'Accept & remember' });
  console.log('Accept & remember button count:', await rememberBtn.count());
  if (await rememberBtn.count() > 0) {
    await rememberBtn.first().click();
    console.log('clicked Accept & remember');
  }

  // Wait and see if a second identical approval appears (it should NOT, if remember works).
  let sawSecondApproval = false;
  for (let i = 0; i < 20; i++) {
    await app.page.waitForTimeout(1000);
    const count = await app.page.locator('text=Approval required').count();
    if (count > 0) { sawSecondApproval = true; break; }
    const text2 = await app.page.evaluate(() => document.body.innerText);
    if (/Run ended/i.test(text2)) break;
  }
  console.log('saw a second approval banner after "remember":', sawSecondApproval);
  await app.shot('qa-sweep-agents-run1-after-remember');
}

await app.done();
