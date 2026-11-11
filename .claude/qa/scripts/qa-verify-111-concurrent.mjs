// Independent re-verification of QA-111's secondary effect: the old
// service-wide `runs` lock held across kill()'s blocking wait() meant
// *other* runs' event pumps could stall while one run was being killed.
// Tauri exposes a single page per window (confirmed in app.mjs: it grabs
// contexts()[0].pages()[0], there is no second page to open), so run A is
// driven through the UI (the one the user actually clicks) and run B is
// started directly via invoke on the same page — same backend service,
// same shared `runs` lock, which is exactly what this bug was about. While
// A is being killed via the UI's "End run", poll B's raw event count to
// check for a stall.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

const app = await open('agents/run');
const { page } = app;

let runIdA = null;
page.on('response', async (res) => {
  if (res.url().includes('agents_start_run') && !runIdA) {
    try {
      runIdA = JSON.parse(await res.text());
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

for (let i = 0; i < 20 && !runIdA; i++) await page.waitForTimeout(500);
console.log('runIdA:', runIdA);

const backends = await page.evaluate(async () =>
  window.__TAURI_INTERNALS__.invoke('agents_list_backends', {})
);
const backendId = backends[0]?.id;
console.log('backends:', JSON.stringify(backends), '-> using', backendId);

// Start run B directly via invoke (bypasses the UI's single-run-at-a-time
// form, but hits the exact same AgentRunService/runs-lock backend path).
const runIdB = await page.evaluate(
  async ({ cwd, backendId }) =>
    window.__TAURI_INTERNALS__.invoke('agents_start_run', {
      backendId,
      cwd,
      prompt: 'Count slowly from 1 to 20, one number per message, pausing briefly between each.',
      model: null,
      resumeSessionId: null,
    }),
  { cwd: CWD, backendId }
);
console.log('runIdB:', runIdB);

async function eventCount(runId) {
  const events = await page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
    runId
  );
  return events.length;
}

// Let both runs get going for a couple seconds.
await page.waitForTimeout(3000);
console.log('run A events before cancel:', await eventCount(runIdA));
console.log('run B events before cancel:', await eventCount(runIdB));

// Poll B's event count in the background while we cancel A via the UI.
const bCounts = [];
let polling = true;
const pollerDone = (async () => {
  while (polling) {
    bCounts.push({ t: Date.now(), n: await eventCount(runIdB) });
    await page.waitForTimeout(200);
  }
})();

console.log('clicking End run (on run A, the UI-driven one) at', new Date().toISOString());
const tClick0 = Date.now();
await page.getByRole('button', { name: 'End run' }).click();

let aExited = false;
const tStart = Date.now();
while (Date.now() - tStart < 6000) {
  await page.waitForTimeout(300);
  if (!aExited) {
    const evs = await page.evaluate(
      async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
      runIdA
    );
    if (evs.some((e) => e.type === 'exited')) {
      aExited = true;
      console.log('run A exited event seen at +', Date.now() - tClick0, 'ms');
    }
  }
}
polling = false;
await pollerDone;

console.log('run A exited during window:', aExited);
console.log('run B event counts over time:', bCounts.map((c) => c.n).join(','));

let maxStallMs = 0;
let stallStart = null;
for (let i = 1; i < bCounts.length; i++) {
  if (bCounts[i].n === bCounts[i - 1].n) {
    if (stallStart === null) stallStart = bCounts[i - 1].t;
    maxStallMs = Math.max(maxStallMs, bCounts[i].t - stallStart);
  } else {
    stallStart = null;
  }
}
console.log('longest stretch with no new events on run B:', maxStallMs, 'ms');
console.log('run B total growth during window (last - first):', bCounts.at(-1).n - bCounts[0].n);

await app.shot('qa-verify-111-concurrent-after');

// Clean up run B too.
try {
  await page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_cancel', { runId: rid }),
    runIdB
  );
} catch {
  /* best effort */
}

await app.done();
