import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

let runId = null;
page.on('response', async (res) => {
  if (res.url().includes('agents_start_run')) {
    try {
      runId = JSON.parse(await res.text());
      console.log('run_id:', runId);
    } catch {
      /* ignore */
    }
  }
});

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page.getByPlaceholder('What should the agent do?').fill('Say hello and wait.');
await page.getByRole('button', { name: 'Start run' }).click();

for (let i = 0; i < 20; i++) {
  await page.waitForTimeout(500);
  if ((await page.getByRole('button', { name: 'End run' }).count()) > 0) break;
}
await page.getByRole('button', { name: 'End run' }).click();
console.log('clicked End run at', new Date().toISOString());

for (let i = 0; i < 120; i++) {
  await page.waitForTimeout(500);
  const ended = (await page.getByText('Run ended').count()) > 0;
  if (ended) {
    console.log('UI shows Run ended after', (i + 1) * 0.5, 's');
    break;
  }
  if (i === 119) console.log('UI NEVER showed Run ended after 60s');
}
await app.shot('qa-sweep2-cancel-patient-final');
await app.done();
