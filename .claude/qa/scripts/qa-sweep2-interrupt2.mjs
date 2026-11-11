import { open } from './app.mjs';

const GOOD_CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;
await page.getByPlaceholder('Pick a folder…').fill(GOOD_CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Count slowly from 1 to 5, explaining each number in a full paragraph.');
await page.getByRole('button', { name: 'Start run' }).click();

let sawStop = false;
for (let i = 0; i < 30; i++) {
  await page.waitForTimeout(300);
  if ((await page.getByRole('button', { name: 'Stop' }).count()) > 0) {
    sawStop = true;
    break;
  }
}
console.log('saw Stop button while running:', sawStop);
await app.shot('qa-sweep2-interrupt2-running');

if (sawStop) {
  await page.getByRole('button', { name: 'Stop' }).click();
  console.log('clicked Stop');
  let sawSendAgain = false;
  for (let i = 0; i < 20; i++) {
    await page.waitForTimeout(500);
    if ((await page.getByRole('button', { name: 'Send' }).count()) > 0) {
      sawSendAgain = true;
      break;
    }
  }
  console.log('turn stopped, composer back to Send:', sawSendAgain);
  await app.shot('qa-sweep2-interrupt2-after-stop');
}
await app.done();
