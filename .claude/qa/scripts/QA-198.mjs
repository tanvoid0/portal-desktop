// QA-198: a terminal turn never reaches a Claude Code thread that was resumed in Portal.
//   CDP_PORT=9223 node .claude/qa/scripts/QA-198.mjs
// Spends three one-word haiku calls. Leaves one scratch session (id printed) in %TEMP%.
import { execFileSync } from 'node:child_process';
import { mkdtempSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { join } from 'node:path';
import { open } from './app.mjs';

const cwd = mkdtempSync(join(tmpdir(), 'qa-198-'));
const env = Object.fromEntries(
  Object.entries(process.env).filter(([k]) => !/^(CLAUDE|ANTHROPIC_BASE_URL)/.test(k))
);
const claude = (...args) =>
  JSON.parse(
    execFileSync(
      join(homedir(), '.local/bin/claude'),
      ['-p', ...args, '--model', 'haiku', '--output-format', 'json'],
      { cwd, env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }
    )
  );

const app = await open('');
const call = (c, a) =>
  app.page.evaluate(([c, a]) => window.__TAURI_INTERNALS__.invoke(c, a), [c, a]);
const wait = (s) => app.page.waitForTimeout(s * 1000);

const id = claude('reply with the single word alpha').session_id;
console.log('session', id, 'cwd', cwd);
await wait(45); // 30s transcript sweep + 10s settle
await call('agents_resume_session', { runId: id, text: 'reply beta', commandId: crypto.randomUUID() });
await wait(25);
await call('agents_cancel', { runId: id });
await wait(5);
claude('--resume', id, 'reply gamma');
await wait(45);

const rows = await call('agents_get_session', { sessionId: id });
const thread = await call('coder_thread_snapshot', { threadId: id });
const inRows = rows.messages.some((m) => m.text === 'reply gamma');
const inThread = JSON.stringify(thread.state).includes('reply gamma') ||
  JSON.stringify(thread.messages ?? []).includes('reply gamma');
console.log('gamma in synced rows:', inRows, '| gamma in thread:', inThread);
console.log('agent_events surviving the sweep replace:', rows.events.length);
console.log(inRows && !inThread ? 'REPRODUCED' : 'not reproduced');
await app.done();
