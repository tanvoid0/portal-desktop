// Instrumented: captures the run_id from agents_start_run's IPC response, then
// polls agents_get_events fast right after clicking Start, to see every event
// type that actually arrives (in particular: does `permission_request` ever
// fire for a Bash tool call, even fleetingly).
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

let runId = null;
page.on('response', async (res) => {
  if (res.url().includes('agents_start_run')) {
    try {
      const body = await res.text();
      console.log('agents_start_run raw response:', body);
      const parsed = JSON.parse(body);
      runId = parsed?.result ?? parsed;
    } catch (e) {
      console.log('could not parse agents_start_run response:', e.message);
    }
  }
});

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Run the shell command `echo qa-marker-99999` using your Bash tool and show me the output.');
await page.getByRole('button', { name: 'Start run' }).click();

// Poll fast for a few seconds to catch the event stream as it happens.
const seenTypes = new Set();
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(250);
  if (!runId) continue;
  try {
    const events = await page.evaluate(
      async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
      runId
    );
    for (const e of events) seenTypes.add(e.type);
  } catch {
    // ignore transient
  }
}
console.log('run_id:', runId);
console.log('event types seen over the run:', [...seenTypes]);

await app.shot('qa-sweep2-raw-events-final');
await app.done();
