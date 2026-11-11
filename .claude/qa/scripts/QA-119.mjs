// QA-119: reloading (or restarting) while a claude-cli tool call is pending approval
// silently drops the live "Approval required" banner. The thread then renders as
// "Running… Thinking…" forever — misleading, since the backend run is actually stuck
// waiting on a decision nobody can give it anymore (the original tool call is lost:
// never approved, never rejected). The only way out is "Stop agent" (still visible and
// still works after reload — confirmed in qa-check-stop-after-reload.mjs — but nothing
// tells the user that's what's wrong, or that it's needed). If the user never revisits
// the thread, the backend run (and its `claude` subprocess) leaks for the rest of the
// app session. Separately: no Stop control is reachable at all while the banner is
// showing in the first place (composer swaps back to Send, not Stop, because `running`
// is forced false while a decision is pending) — so there is no way to abort a turn
// mid-approval without first Accepting or Rejecting that one tool call.
//
// Needs a workspace folder to run `claude` in — point WORKSPACE at any folder you can
// write a throwaway file into (defaults to this repo's own QA fixtures dir).
import { mkdirSync } from 'node:fs';
import { open } from './app.mjs';

const FIXTURE =
  process.env.WORKSPACE ??
  'D:\\projects\\production\\portal\\portal_desktop\\.claude\\qa\\fixtures\\QA claude-cli engine test';
mkdirSync(FIXTURE, { recursive: true }); // repro creates its own scratch workspace if missing

const app = await open('ai/coder');
const p = app.page;
await p.waitForTimeout(500);

async function invoke(cmd, args) {
  return p.evaluate(async ({ cmd, args }) => window.__TAURI_INTERNALS__.invoke(cmd, args), { cmd, args });
}

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
await textarea.fill(
  'Use your Bash tool to run this exact command: echo qa119 > qa-119-marker.txt -- then tell me the file name you created.'
);
await p.keyboard.press('Control+Enter');

const banner = p.getByText('Approval required');
await banner.waitFor({ timeout: 30000 });
console.log('1. banner is up:', true);
await app.shot('QA-119-before-banner-up-no-stop-control');

// No Stop control reachable while the banner shows — Stop swaps back to Send.
const stopVisible = await p.getByTitle('Stop agent').count();
console.log('2. Stop agent button present while banner is up (expect 0):', stopVisible);

// Full reload — simulates the app restarting or the page being refreshed.
await p.reload();
await p.waitForLoadState('load');
await p.waitForTimeout(1500);
const bannerAfterReload = await banner.count();
console.log('3. banner present after reload (expect 0 — silently dropped):', bannerAfterReload);
await app.shot('QA-119-before-after-reload-banner-gone');

const stillActive = await invoke('coder_list_running', {});
console.log(
  '4. backend run for this thread still active after reload (expect true — orphaned):',
  stillActive.includes(thread.id)
);

// Cleanup: only `coder_stop` called directly can end it now — nothing in the UI can.
await invoke('coder_stop', { threadId: thread.id });
await p.waitForTimeout(1000);
console.log('cleanup stop result:', !(await invoke('coder_list_running', {})).includes(thread.id));

await app.done();
