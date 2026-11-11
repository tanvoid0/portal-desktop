import { open } from './app.mjs';

const BAD_CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA-nonexistent-dir-12345\\nope';
const GOOD_CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

// --- Part 1: bad cwd ---
{
  const app = await open('agents/run');
  const { page } = app;
  await page.getByPlaceholder('Pick a folder…').fill(BAD_CWD);
  await page.getByPlaceholder('What should the agent do?').fill('Say hello.');
  await page.getByRole('button', { name: 'Start run' }).click();

  let sawError = false;
  let stuckStarting = false;
  for (let i = 0; i < 30; i++) {
    await page.waitForTimeout(1000);
    if ((await page.locator('.text-destructive').count()) > 0) {
      sawError = true;
      break;
    }
  }
  stuckStarting = (await page.getByRole('button', { name: 'Starting…' }).count()) > 0;
  console.log('[badcwd] saw error message:', sawError);
  console.log('[badcwd] button stuck on Starting…:', stuckStarting);
  await app.shot('qa-sweep2b-badcwd-state');
  await app.done();
}

// --- Part 2: interrupt (Stop button) mid-run ---
{
  const app = await open('agents/run');
  const { page } = app;
  await page.getByPlaceholder('Pick a folder…').fill(GOOD_CWD);
  await page
    .getByPlaceholder('What should the agent do?')
    .fill('Count slowly from 1 to 5, explaining each number in a full paragraph.');
  await page.getByRole('button', { name: 'Start run' }).click();

  let sawStop = false;
  for (let i = 0; i < 20; i++) {
    await page.waitForTimeout(300);
    if ((await page.getByRole('button', { name: 'Stop' }).count()) > 0) {
      sawStop = true;
      break;
    }
  }
  console.log('[interrupt] saw Stop button while running:', sawStop);
  if (sawStop) {
    await page.getByRole('button', { name: 'Stop' }).click();
    console.log('[interrupt] clicked Stop');
    let sawSendAgain = false;
    for (let i = 0; i < 20; i++) {
      await page.waitForTimeout(500);
      if ((await page.getByRole('button', { name: 'Send' }).count()) > 0) {
        sawSendAgain = true;
        break;
      }
    }
    console.log('[interrupt] turn stopped, composer back to Send:', sawSendAgain);
  }
  await app.shot('qa-sweep2b-interrupt-state');
  await app.done();
}
