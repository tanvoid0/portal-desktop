// Start a run, click "End run" mid-run, confirm the UI flips to the ended
// state and (checked separately via process list) the child `claude` process
// actually gets killed, not orphaned.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page.getByPlaceholder('What should the agent do?').fill('Say hello and wait.');
await page.getByRole('button', { name: 'Start run' }).click();

let sawRunning = false;
for (let i = 0; i < 20; i++) {
  await page.waitForTimeout(500);
  if ((await page.getByRole('button', { name: 'End run' }).count()) > 0) {
    sawRunning = true;
    break;
  }
}
console.log('saw End run button:', sawRunning);
await app.shot('qa-sweep2-endrun-before');

if (sawRunning) {
  await page.getByRole('button', { name: 'End run' }).click();
  console.log('clicked End run');
  let sawEnded = false;
  for (let i = 0; i < 15; i++) {
    await page.waitForTimeout(500);
    if ((await page.getByText('Run ended').count()) > 0) {
      sawEnded = true;
      break;
    }
  }
  console.log('UI shows Run ended:', sawEnded);
  await app.shot('qa-sweep2-endrun-after');

  if (sawEnded) {
    const newRunVisible = (await page.getByRole('button', { name: 'New run' }).count()) > 0;
    console.log('New run button visible:', newRunVisible);
    await page.getByRole('button', { name: 'New run' }).click();
    await page.waitForTimeout(500);
    const backToForm = (await page.getByRole('button', { name: 'Start run' }).count()) > 0;
    console.log('New run returns to launcher form:', backToForm);
    await app.shot('qa-sweep2-newrun-after');
  }
}

await app.done();
