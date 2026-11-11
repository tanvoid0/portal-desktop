// Same instrumentation as qa-sweep2-raw-events.mjs but asks for a Write tool
// call (creating a new file) instead of Bash echo — Write is the tool type
// least likely to ever be pre-approved by any built-in CLI heuristic.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

let runId = null;
page.on('response', async (res) => {
  if (res.url().includes('agents_start_run')) {
    try {
      const body = await res.text();
      runId = JSON.parse(body);
      console.log('run_id:', runId);
    } catch {
      /* ignore */
    }
  }
});

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill(
    'Use your Write tool to create a new file called qa-write-test.txt in this directory with the content "hello from qa".'
  );
await page.getByRole('button', { name: 'Start run' }).click();

const seenTypes = new Set();
const permissionEvents = [];
for (let i = 0; i < 60; i++) {
  await page.waitForTimeout(250);
  if (!runId) continue;
  try {
    const events = await page.evaluate(
      async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
      runId
    );
    for (const e of events) {
      seenTypes.add(e.type);
      if (e.type === 'permission_request' || e.type === 'permission_resolved') {
        permissionEvents.push(e);
      }
    }
  } catch {
    /* transient */
  }
}
console.log('event types seen over the run:', [...seenTypes]);
console.log('permission events:', JSON.stringify(permissionEvents, null, 2));
await app.shot('qa-sweep2-write-test-final');
await app.done();
