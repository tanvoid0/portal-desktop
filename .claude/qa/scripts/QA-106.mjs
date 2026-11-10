// QA-106 repro: the Agents → Run launcher's backend `Select` renders with no
// options and no error feedback whenever `agentBackendsQuery` fails, instead of
// surfacing the query error. Coder: set CDP_PORT to whatever dev instance you're
// using (default 9222).
//
//   CDP_PORT=9223 node .claude/qa/scripts/QA-106.mjs
import { open } from './app.mjs';

const app = await open('agents/run');
await app.page.waitForTimeout(2000);

await app.page.locator('#agent-backend').click();
await app.page.waitForTimeout(400);

const optionCount = await app.page.locator('[role=option]').count();
const pageText = await app.page.evaluate(() => document.body.innerText);
const mentionsError = /fail|error|try again|retry/i.test(pageText);

console.log(`backend options rendered: ${optionCount}`);
console.log(`page mentions an error/retry anywhere: ${mentionsError}`);
if (optionCount === 0 && !mentionsError) {
  console.log('REPRO: empty select, zero user-visible feedback that the backend list failed to load.');
} else {
  console.log('Not reproduced as described — recheck manually.');
}

await app.shot('QA-106-after-check');
await app.done();
