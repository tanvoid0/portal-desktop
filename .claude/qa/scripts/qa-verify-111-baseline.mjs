// Baseline for qa-verify-111-concurrent.mjs: same "count slowly" prompt,
// single run, no cancel involved — to see what gaps between events are
// normal model pacing vs. what the concurrent test saw after a cancel.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

let runId = null;
page.on('response', async (res) => {
  if (res.url().includes('agents_start_run') && !runId) {
    try {
      runId = JSON.parse(await res.text());
    } catch {
      /* ignore */
    }
  }
});

await page.getByPlaceholder('Pick a folder…').fill(CWD);
await page
  .getByPlaceholder('What should the agent do?')
  .fill('Count slowly from 1 to 20, one number per message, pausing briefly between each.');
await page.getByRole('button', { name: 'Start run' }).click();

for (let i = 0; i < 20 && !runId; i++) await page.waitForTimeout(500);
console.log('runId:', runId);

const counts = [];
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(250);
  const events = await page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
    runId
  );
  counts.push({ t: Date.now(), n: events.length });
}
console.log('event counts over 10s, no cancel:', counts.map((c) => c.n).join(','));

let maxStallMs = 0;
let stallStart = null;
for (let i = 1; i < counts.length; i++) {
  if (counts[i].n === counts[i - 1].n) {
    if (stallStart === null) stallStart = counts[i - 1].t;
    maxStallMs = Math.max(maxStallMs, counts[i].t - stallStart);
  } else {
    stallStart = null;
  }
}
console.log('longest gap with no new events (baseline, no cancel):', maxStallMs, 'ms');

try {
  await page.getByRole('button', { name: 'End run' }).click({ timeout: 2000 });
} catch {
  /* ignore */
}
await app.done();
