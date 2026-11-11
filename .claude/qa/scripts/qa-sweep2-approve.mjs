// QA sweep: Agents -> Run. Start a run that triggers a tool call (Read), confirm the
// approval banner appears, Approve it, confirm the run continues.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Read the file qa-note.txt in this directory and tell me what it says.');
await page.getByRole('button', { name: 'Start run' }).click();

console.log('clicked Start run, waiting for approval banner or run end (up to 45s)...');

let sawApproval = false;
let sawEnded = false;
let sawError = false;
for (let i = 0; i < 45; i++) {
  await page.waitForTimeout(1000);
  const acceptOnce = await page.getByRole('button', { name: 'Accept once' }).count();
  const ended = await page.getByText('Run ended').count();
  const errBox = await page.locator('.text-destructive').count();
  if (acceptOnce > 0) {
    sawApproval = true;
    break;
  }
  if (ended > 0) {
    sawEnded = true;
    break;
  }
  if (errBox > 0) {
    sawError = true;
    break;
  }
}

console.log('saw approval banner:', sawApproval);
console.log('saw run ended (no approval needed):', sawEnded);
console.log('saw error box:', sawError);
await app.shot('qa-sweep2-approve-state');

if (sawApproval) {
  await page.getByRole('button', { name: 'Accept once' }).click();
  console.log('clicked Accept once, waiting up to 30s for progress...');
  await page.waitForTimeout(3000);
  await app.shot('qa-sweep2-after-accept');
  for (let i = 0; i < 27; i++) {
    const ended = await page.getByText('Run ended').count();
    if (ended > 0) break;
    await page.waitForTimeout(1000);
  }
  await app.shot('qa-sweep2-after-accept-settled');
}

await app.done();
