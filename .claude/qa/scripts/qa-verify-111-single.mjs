// Independent re-verification of QA-111 (single run): start a run, click
// "End run", confirm the header flips to "Run ended" / "New run" promptly,
// and confirm the backend actually emits an `exited` event (not just a UI
// trick) within a tight window.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

let runId = null;
page.on('response', async (res) => {
  if (res.url().includes('agents_start_run')) {
    try {
      runId = JSON.parse(await res.text());
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

const t0 = Date.now();
await page.getByRole('button', { name: 'End run' }).click();
console.log('clicked End run at', new Date().toISOString());

let sawExited = false;
let exitedMs = null;
let uiEndedMs = null;
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(250);
  if (!sawExited && runId) {
    const events = await page.evaluate(
      async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
      runId
    );
    if (events.some((e) => e.type === 'exited')) {
      sawExited = true;
      exitedMs = Date.now() - t0;
    }
  }
  if (uiEndedMs === null && (await page.getByText('Run ended').count()) > 0) {
    uiEndedMs = Date.now() - t0;
  }
  if (sawExited && uiEndedMs !== null) break;
}

console.log('backend emitted exited event:', sawExited, exitedMs !== null ? `(+${exitedMs}ms)` : '');
console.log('UI shows "Run ended":', uiEndedMs !== null, uiEndedMs !== null ? `(+${uiEndedMs}ms)` : '');
const hasNewRunBtn = (await page.getByRole('button', { name: 'New run' }).count()) > 0;
console.log('"New run" button present:', hasNewRunBtn);
await app.shot('qa-verify-111-single-after-endrun');
await app.done();
