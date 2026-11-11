// Checks whether a destructive Bash command (file removal) triggers
// permission_request, to disambiguate "Bash isn't gated at all" from
// "trivial echo is pre-approved, destructive ops still gate correctly".
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
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Use your Bash tool to run `rm -f qa-note.txt` in this directory.');
await page.getByRole('button', { name: 'Start run' }).click();

const seenTypes = new Set();
let permCount = 0;
for (let i = 0; i < 50; i++) {
  await page.waitForTimeout(250);
  if (!runId) continue;
  try {
    const events = await page.evaluate(
      async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
      runId
    );
    for (const e of events) {
      seenTypes.add(e.type);
      if (e.type === 'permission_request') permCount++;
    }
  } catch {
    /* transient */
  }
}
console.log('event types seen:', [...seenTypes]);
console.log('saw permission_request:', permCount > 0);
await app.shot('qa-sweep2-bash-destructive-final');
await app.done();
