// QA sweep: Agents -> Run. Try a Bash tool call (more likely to require
// per-call approval than a Read, which may already be allow-listed in the
// user's own Claude settings) to catch the approval banner.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Run the shell command `echo qa-marker-12345` using your Bash tool and show me the output.');
await page.getByRole('button', { name: 'Start run' }).click();

console.log('clicked Start run, waiting for approval banner or run end (up to 45s)...');

let sawApproval = false;
let sawEnded = false;
for (let i = 0; i < 45; i++) {
  await page.waitForTimeout(1000);
  const acceptOnce = await page.getByRole('button', { name: 'Accept once' }).count();
  const ended = await page.getByText('Run ended').count();
  if (acceptOnce > 0) {
    sawApproval = true;
    break;
  }
  if (ended > 0) {
    sawEnded = true;
    break;
  }
}
console.log('saw approval banner:', sawApproval, 'at iteration');
console.log('saw run ended:', sawEnded);
await app.shot('qa-sweep2-approve2-state');

if (sawApproval) {
  // Deny this time.
  await page.getByRole('button', { name: 'Reject' }).click();
  console.log('clicked Reject, waiting up to 20s...');
  await page.waitForTimeout(3000);
  await app.shot('qa-sweep2-after-reject');
  for (let i = 0; i < 17; i++) {
    const ended = await page.getByText('Run ended').count();
    if (ended > 0) break;
    await page.waitForTimeout(1000);
  }
  await app.shot('qa-sweep2-after-reject-settled');
}

await app.done();
