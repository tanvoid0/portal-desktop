// #283: claude-cli approval banner (1) showed no command text, (2) Reject left the
// buttons disabled and the agent never resumed. Verifies both live.
import { mkdirSync } from 'node:fs';
import { open } from './app.mjs';

const FIXTURE =
  process.env.WORKSPACE ??
  'D:\\projects\\production\\portal\\portal_desktop\\.claude\\qa\\fixtures\\QA claude-cli engine test';
mkdirSync(FIXTURE, { recursive: true });
const CMD = 'echo qa283 > qa-283-marker.txt';

const app = await open('ai/coder');
const p = app.page;
await p.waitForTimeout(500);
const invoke = (cmd, args) =>
  p.evaluate(async ({ cmd, args }) => window.__TAURI_INTERNALS__.invoke(cmd, args), { cmd, args });

await invoke('coder_set_permission_mode', { mode: 'review' });
const thread = await invoke('coder_create_thread', {
  workspaceRoot: FIXTURE,
  model: null,
  llmProvider: 'claude-cli',
  threadKind: 'session',
  projectId: null,
});
console.log('thread', thread.id);
const origin = new URL(p.url()).origin;
await p.goto(`${origin}/ai/coder?id=${thread.id}`);
await p.waitForLoadState('load');
await p.waitForTimeout(1200);

const textarea = p.getByPlaceholder(/Ask the coder|Send follow-up/).first();
await textarea.click();
await textarea.fill(`Use your Bash tool to run this exact command: ${CMD} -- then tell me what happened.`);
await p.keyboard.press('Control+Enter');

await p.getByText('Approval required').waitFor({ timeout: 60000 });
const bannerText = await p.locator('div.border-amber-500\\/50').first().innerText();
console.log('1. banner text:', JSON.stringify(bannerText));
console.log('   shows command (expect true):', bannerText.includes('qa283'));
await app.shot('QA-283-banner');

const reject = p.locator('button').filter({ hasText: /^Reject$/ });
console.log('2. Reject enabled before click (expect true):', await reject.isEnabled());
await reject.click();

// Banner must go away and the run must finish or ask again — not hang.
const t0 = Date.now();
let state = 'hung';
while (Date.now() - t0 < 90000) {
  await p.waitForTimeout(1000);
  const banners = await p.getByText('Approval required').count();
  const running = (await invoke('coder_list_running', {})).includes(thread.id);
  if (banners === 0 && !running) { state = 'finished'; break; }
  if (banners > 0 && (await p.locator('button').filter({ hasText: /^Reject$/ }).isEnabled())) {
    state = 'asked-again(enabled)'; break;
  }
}
console.log('3. after Reject:', state, `after ${Math.round((Date.now() - t0) / 1000)}s`);
const bannersLeft = await p.getByText('Approval required').count();
console.log('   banner still showing:', bannersLeft);
await app.shot('QA-283-after-reject');

await invoke('coder_stop', { threadId: thread.id }).catch(() => {});
await app.done();
