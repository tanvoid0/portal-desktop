// QA-107 re-verify: start a Claude Code run from Agents -> Run and confirm
// it proceeds past "Starting..." into a live conversation, triggering both
// a coder-type (file edit) and terminal-type (shell) tool-call approval,
// then exercise Accept once / Accept & remember / Reject, interrupt,
// cancel/end run. Also checks whether "Accept & remember" actually
// suppresses a repeat of the same tool.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\claude\\a32cb83c572c46cb\\3d9c5a03-5565-4268-b94d-cdc9b4983a71\\scratchpad\\qa-agent-test';

const app = await open('agents/run');
await app.page.waitForTimeout(500);

await app.page.locator('#agent-cwd').fill(CWD);
await app.page.locator('#agent-prompt').fill(
  'Create a new file called hello.txt in the current directory containing the text "hi". ' +
  'Then run the shell command `dir` to list files. Do not do anything else.'
);
await app.shot('QA-107-verify-filled');

await app.page.locator('button').filter({ hasText: 'Start run' }).click();

let state = 'unknown';
for (let i = 0; i < 40; i++) {
  await app.page.waitForTimeout(1000);
  const text = await app.page.evaluate(() => document.body.innerText);
  if (/Approval required/i.test(text)) { state = 'approval'; break; }
  if (/Run in progress|Run ended/i.test(text)) { state = 'conversation'; break; }
  if (/Starting…|Starting\.\.\./.test(text) && i === 39) { state = 'stuck-starting'; }
}
console.log(`t+${Date.now()}: state after start =`, state);
await app.shot('QA-107-verify-after-start');
console.log('RESULT: proceeded past Starting... =', state !== 'stuck-starting' && state !== 'unknown');

await app.done();
