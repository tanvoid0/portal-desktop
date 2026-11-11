// QA-120: cancelling a claude-cli run only kills the `cmd /C claude` wrapper, not the
// real `claude.exe` grandchild process it started — the wrapper exits, the actual CLI
// process (and whatever workspace cwd it holds open) keeps running indefinitely.
//
// This script starts a claude-cli/Review thread, waits for a live permission request
// (so the process is guaranteed to still be alive and blocked), cancels it through
// Portal, then checks the OS process list for a lingering `claude.exe ... portal-agents`
// process. Needs PowerShell (Get-CimInstance) on the PATH, i.e. Windows.
import { open } from './app.mjs';
import { execSync } from 'node:child_process';

const FIXTURE =
  process.env.WORKSPACE ??
  'D:\\projects\\production\\portal\\portal_desktop\\.claude\\qa\\fixtures\\QA claude-cli engine test';

function listPortalClaudeProcesses() {
  const out = execSync(
    `powershell -Command "Get-CimInstance Win32_Process -Filter \\"Name='claude.exe'\\" | Where-Object { $_.CommandLine -like '*portal-agents*' } | Select-Object -ExpandProperty ProcessId"`,
    { encoding: 'utf8' }
  );
  return out.split(/\s+/).map((s) => s.trim()).filter(Boolean);
}

const before = new Set(listPortalClaudeProcesses());
console.log('portal-spawned claude.exe PIDs before:', [...before]);

const app = await open('ai/coder');
const p = app.page;
await p.waitForTimeout(500);
async function invoke(cmd, args) {
  return p.evaluate(async ({ cmd, args }) => window.__TAURI_INTERNALS__.invoke(cmd, args), { cmd, args });
}
await invoke('coder_set_permission_mode', { mode: 'review' });
const thread = await invoke('coder_create_thread', {
  workspaceRoot: FIXTURE, model: null, llmProvider: 'claude-cli', threadKind: 'session', projectId: null,
});
console.log('thread', thread.id);

await invoke('coder_send', {
  threadId: thread.id,
  message:
    'Use your Bash tool to run this exact command: echo qa120 > qa-120-marker.txt -- then tell me the file name you created.',
});

// Poll until the run shows up as active (process spawned + pending request reached).
for (let i = 0; i < 20; i++) {
  await p.waitForTimeout(1000);
  const running = await invoke('coder_list_running', {});
  if (running.includes(thread.id)) break;
}
await p.waitForTimeout(2000); // give the hook relay a moment past "running" to actually reach the tool call

console.log('--- cancelling now ---');
await invoke('coder_stop', { threadId: thread.id });
await p.waitForTimeout(1000);
console.log('active runs after stop (expect empty):', JSON.stringify(await invoke('coder_list_running', {})));

const after = listPortalClaudeProcesses();
const newPids = after.filter((pid) => !before.has(pid));
console.log('portal-spawned claude.exe PIDs after stop:', after);
console.log('NEW leaked PID(s) from this run (expect none):', newPids);

await app.done();
if (newPids.length) {
  console.log(`LEAK CONFIRMED: ${newPids.join(', ')} still running — Portal only killed the cmd.exe wrapper.`);
  console.log('Clean up manually: taskkill /PID <pid> /F (or Task Manager).');
}
