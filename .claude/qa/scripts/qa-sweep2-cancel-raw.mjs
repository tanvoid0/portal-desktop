// Instrumented: start a run, call cancel() via UI's End run, then poll raw
// backend events for an `exited` event, with a generous timeout, to see
// whether the backend actually emits it (decoupling backend-cancel-works
// from frontend-shows-ended).
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

let sawExited = false;
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(500);
  if (!runId) continue;
  const events = await page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
    runId
  );
  if (events.some((e) => e.type === 'exited')) {
    sawExited = true;
    console.log('saw exited event at', new Date().toISOString(), 'after', (i + 1) * 0.5, 's');
    break;
  }
}
console.log('backend emitted exited event:', sawExited);

// Check if the UI itself reflects it by now.
const uiShowsEnded = (await page.getByText('Run ended').count()) > 0;
console.log('UI shows Run ended text:', uiShowsEnded);
await app.shot('qa-sweep2-cancel-raw-final');

await app.done();
