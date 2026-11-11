// End-to-end: a destructive Bash command should trigger the approval banner
// in the real UI. Click "Accept once", confirm the run proceeds (tool
// executes, turn completes).
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Use your Bash tool to run `rm -f qa-approve-marker.txt` in this directory, then tell me you did it.');
await page.getByRole('button', { name: 'Start run' }).click();

let sawApproval = false;
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(500);
  if ((await page.getByRole('button', { name: 'Accept once' }).count()) > 0) {
    sawApproval = true;
    break;
  }
}
console.log('saw approval banner (Accept once visible):', sawApproval);
await app.shot('qa-sweep2-approve-final-banner');

if (sawApproval) {
  await page.getByRole('button', { name: 'Accept once' }).click();
  console.log('clicked Accept once');
  let completed = false;
  for (let i = 0; i < 30; i++) {
    await page.waitForTimeout(1000);
    const acceptGone = (await page.getByRole('button', { name: 'Accept once' }).count()) === 0;
    const msgCount = await page.locator('text=did it').count();
    if (acceptGone && msgCount > 0) {
      completed = true;
      break;
    }
  }
  console.log('run continued after approval (saw follow-up text):', completed);
  await app.shot('qa-sweep2-approve-final-after-accept');
}

await app.done();
