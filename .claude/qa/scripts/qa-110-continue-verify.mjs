// QA-110 + QA-112 verification: start a run that needs a destructive Bash
// tool, click through every sequential "Accept once" that appears (the
// agent may re-invoke Bash more than once in the same turn), confirm each
// banner has no "Accept & remember" (QA-112), and confirm the turn
// eventually completes. Single `open()` session throughout — `open()`
// navigates the page, which resets the run launcher's in-memory state, so
// everything has to happen in one script run, not split across processes.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill(
    'Use your Bash tool to run `rm -f qa-approve-marker.txt` in this directory, then tell me you did it.'
  );
await page.getByRole('button', { name: 'Start run' }).click();

let rememberSeenAnywhere = false;
let completed = false;
outer: for (let round = 0; round < 5; round++) {
  for (let i = 0; i < 60; i++) {
    await page.waitForTimeout(500);
    if ((await page.locator('text=did it').count()) > 0) {
      completed = true;
      break outer;
    }
    if ((await page.getByRole('button', { name: 'Accept once' }).count()) > 0) break;
  }
  const hasAccept = (await page.getByRole('button', { name: 'Accept once' }).count()) > 0;
  if (!hasAccept) {
    console.log('no approval appeared this round, round', round);
    await app.shot('qa-110-no-approval-moment');
    break;
  }
  const hasRemember = (await page.getByRole('button', { name: 'Accept & remember' }).count()) > 0;
  if (hasRemember) rememberSeenAnywhere = true;
  console.log('round', round, 'approval banner — Accept & remember present:', hasRemember);
  await page.getByRole('button', { name: 'Accept once' }).click();
  console.log('round', round, 'clicked Accept once');
}

if (!completed) {
  for (let i = 0; i < 20; i++) {
    await page.waitForTimeout(1000);
    if ((await page.locator('text=did it').count()) > 0) {
      completed = true;
      break;
    }
  }
}

console.log('turn completed (saw follow-up text):', completed);
console.log('"Accept & remember" ever shown on Agents run:', rememberSeenAnywhere);
await app.shot('qa-110-continue-verify-final');
await app.done();
