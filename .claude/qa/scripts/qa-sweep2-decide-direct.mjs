// The UI's Accept/Reject buttons are disabled while a permission is pending
// (separate bug, filed). This bypasses the UI and calls
// agents_decide_permission directly to check the *backend* decide mechanism
// itself: does approving let the tool proceed, does denying block it and let
// the run recover?
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';
const mode = process.argv[2] === 'deny' ? 'deny' : 'approve';
const marker = mode === 'approve' ? 'qa-approve-direct.txt' : 'qa-deny-direct.txt';

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
await page
  .getByPlaceholder('What should the agent do?')
  .fill(`Use your Write tool to create a file named ${marker} with content "x", then tell me the result.`);
await page.getByRole('button', { name: 'Start run' }).click();

let requestId = null;
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(500);
  if (!runId) continue;
  const events = await page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
    runId
  );
  const pr = events.find((e) => e.type === 'permission_request');
  if (pr) {
    requestId = pr.request_id;
    break;
  }
}
console.log('mode:', mode, 'run_id:', runId, 'request_id:', requestId);
if (!requestId) {
  console.log('FAIL: never saw a permission_request');
  await app.done();
  process.exit(1);
}

await page.evaluate(
  async ({ rid, reqId, allow }) =>
    window.__TAURI_INTERNALS__.invoke('agents_decide_permission', {
      runId: rid,
      requestId: reqId,
      allow,
    }),
  { rid: runId, reqId: requestId, allow: mode === 'approve' }
);
console.log(`called decide_permission(allow=${mode === 'approve'})`);

let endedOrResult = false;
let sawToolResult = false;
let sawFinalMessage = false;
for (let i = 0; i < 30; i++) {
  await page.waitForTimeout(1000);
  const events = await page.evaluate(
    async (rid) => window.__TAURI_INTERNALS__.invoke('agents_get_events', { runId: rid }),
    runId
  );
  sawToolResult = events.some((e) => e.type === 'tool_result');
  sawFinalMessage = events.some((e) => e.type === 'message' && e.role === 'assistant');
  if (sawToolResult || sawFinalMessage) {
    endedOrResult = true;
    break;
  }
}
console.log('saw tool_result after decision:', sawToolResult);
console.log('saw a final assistant message after decision:', sawFinalMessage);
console.log('run continued/recovered after decision:', endedOrResult);
await app.shot(`qa-sweep2-decide-direct-${mode}`);
await app.done();
