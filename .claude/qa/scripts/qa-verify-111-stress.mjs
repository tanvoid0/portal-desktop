// Stress pass aimed at the QA-111 fix agent's unreproduced lead: the whole
// dev-instance backend process exited once (code 1, no panic) during heavy
// concurrent cancel-testing. Repeatedly starts 2-3 runs via invoke and
// cancels them in quick, overlapping succession, several rounds, watching
// whether the backend (CDP port) stays up throughout.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';
const ROUNDS = 8;

const app = await open('agents/run');
const { page } = app;

const backends = await page.evaluate(async () =>
  window.__TAURI_INTERNALS__.invoke('agents_list_backends', {})
);
const backendId = backends[0]?.id;

async function startRun() {
  return page.evaluate(
    async ({ cwd, backendId }) =>
      window.__TAURI_INTERNALS__.invoke('agents_start_run', {
        backendId,
        cwd,
        prompt: 'Say hello and wait.',
        model: null,
        resumeSessionId: null,
      }),
    { cwd: CWD, backendId }
  );
}
async function cancelRun(runId) {
  return page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_cancel', { runId: rid }),
    runId
  );
}

let alive = true;
for (let round = 0; round < ROUNDS && alive; round++) {
  const ids = await Promise.all([startRun(), startRun(), startRun()]);
  await page.waitForTimeout(400);
  // Cancel all three nearly simultaneously, racing each other against the
  // shared service lock.
  try {
    await Promise.all(ids.map((id) => cancelRun(id)));
  } catch (e) {
    console.log(`round ${round}: cancel threw:`, e.message);
  }
  await page.waitForTimeout(200);
  // Health check: can we still invoke at all?
  try {
    await page.evaluate(async () => window.__TAURI_INTERNALS__.invoke('agents_list_backends', {}));
    console.log(`round ${round}: backend responsive, ids=${ids.join(',')}`);
  } catch (e) {
    alive = false;
    console.log(`round ${round}: FAIL backend unresponsive:`, e.message);
  }
}

console.log('backend alive after all rounds:', alive);
await app.done();
