// QA sweep: Agents -> Run with a working directory that does not exist.
// Confirms a sane error surfaces (not a silent hang or crash).
import { open } from './app.mjs';

const BAD_CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA-nonexistent-dir-12345\\nope';

const app = await open('agents/run');
const { page } = app;

await page.getByPlaceholder('Pick a folder…').fill(BAD_CWD);
await page.getByPlaceholder('What should the agent do?').fill('Say hello.');
await page.getByRole('button', { name: 'Start run' }).click();

console.log('clicked Start run with a nonexistent cwd, waiting up to 30s...');

let sawError = false;
let sawRunView = false;
let stuckStarting = false;
for (let i = 0; i < 30; i++) {
  await page.waitForTimeout(1000);
  const errBox = await page.locator('.text-destructive').count();
  const runHeader = await page.getByText('Run in progress').count();
  if (errBox > 0) {
    sawError = true;
    break;
  }
  if (runHeader > 0) {
    sawRunView = true;
    break;
  }
}
const stillStarting = await page.getByRole('button', { name: 'Starting…' }).count();
stuckStarting = stillStarting > 0;

console.log('saw error message:', sawError);
console.log('entered run view anyway:', sawRunView);
console.log('button stuck on Starting…:', stuckStarting);
await app.shot('qa-sweep2-badcwd-state');

await app.done();
