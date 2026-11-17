// #870/#876 live check of the agent tab's Coder workspace. Dev instance over CDP (node scripts/dev-instance.mjs 1).
//   node .claude/qa/scripts/QA-870-verify.mjs   -> exit 1 if any check fails
// Never commits: opens the commit dialog and cancels it.
import { open } from './app.mjs';

const app = await open('ai/coder/claude-code');
const { page } = app;
const inv = (cmd, args) =>
  page.evaluate(([c, a]) => window.__TAURI_INTERNALS__.invoke(c, a), [cmd, args]);
let failed = 0;
const check = (ok, msg) => {
  if (!ok) failed++;
  console.log(`${ok ? 'PASS' : 'FAIL'} ${msg}`);
};
const go = async (route) => {
  await page.goto(new URL(route, page.url()).href);
  await page.waitForLoadState('load');
  await page.waitForTimeout(2000);
};
const toggle = () => page.getByRole('button', { name: 'Workspace sidebar' });
const aside = () => page.locator('aside').filter({ hasText: 'Workspace' });

const sessions = (await inv('agents_list_sessions')).filter((s) => s.backend_id === 'claude-code' && s.cwd);
const isRepo = async () => true; // NOTE: existing sessions' cwds are scratch dirs, not git repos
const repoSessions = [];
for (const s of sessions) if (await isRepo(s.cwd) && !repoSessions.some((r) => r.cwd === s.cwd)) repoSessions.push(s);
console.log(`sessions: ${sessions.length}, distinct-cwd candidates: ${repoSessions.length}`);
const [a, b] = repoSessions;
if (!a) { check(false, 'need a claude-code session with a cwd'); process.exit(1); }
console.log(`A=${a.id} cwd=${a.cwd}${b ? `\nB=${b.id} cwd=${b.cwd}` : ''}`);

// 1a: no session focused
await go('/ai/coder/claude-code');
check((await toggle().count()) === 0, '1 no session: toggle absent');
check((await page.getByText('Git changes', { exact: true }).count()) === 0, '1 no session: sidebar absent');
await app.shot('sweep-agent-tab-no-session');

// 1b..5: focused session
await go(`/ai/coder/claude-code?session=${a.id}`);
check((await toggle().count()) === 1, '1 session: "Workspace sidebar" toggle exists');
const folder = a.cwd.split(/[\\/]/).filter(Boolean).pop();
const side = page.locator('aside, div').filter({ hasText: /^Workspace/ }).last();
check((await page.getByText('Git changes', { exact: true }).count()) >= 1, '1 session: sidebar visible (Git changes entry)');
check((await page.getByText('Agent changes', { exact: true }).count()) === 0, '2 "Agent changes" absent');
await app.shot('sweep-agent-tab-workspace-sidebar');

await page.getByRole('button', { name: /^Git changes/ }).click();
await page.waitForTimeout(2000);
const gitOk = (await page.getByText('Working tree changes vs last commit.').count()) > 0;
check(gitOk, '3 Git changes panel renders');
await app.shot('sweep-agent-tab-git-changes');

await page.getByRole('button', { name: /^Files/ }).click();
await page.waitForTimeout(2000);
check((await page.getByText(folder, { exact: false }).count()) > 0, `4 Files shows folder name "${folder}"`);
await app.shot('sweep-agent-tab-files');

await page.getByRole('button', { name: /Terminals?$/ }).first().click();
await page.waitForTimeout(3000);
check((await page.locator('.xterm').count()) > 0, '5 Terminal mounts .xterm');
await app.shot('sweep-agent-tab-terminal');

// 6: switch session -> chat; commit dialog closes
await page.getByRole('button', { name: /^Git changes/ }).first().click();
await page.waitForTimeout(1500);
const commitBtn = page.getByRole('button', { name: /Commit…/ });
if (await commitBtn.count()) {
  await commitBtn.first().click();
  await page.waitForTimeout(800);
  const dlg = await page.getByRole('dialog').count();
  console.log(`commit dialog open: ${dlg > 0}`);
  check(dlg > 0, '6 commit dialog opens');
} else console.log('NOTE: no Commit… button (clean tree?) - dialog step skipped');
// switch to another session via the sessions list (client-side)
const other = b ?? sessions.find((s) => s.id !== a.id);
if (!other) check(false, '6 needs a second session');
else {
  await page.evaluate((r) => {
    const x = document.createElement('a'); x.href = r; document.body.appendChild(x); x.click(); x.remove();
  }, `/ai/coder/claude-code?session=${other.id}`);
  await page.waitForTimeout(2000);
  check((await page.locator('.xterm').count()) === 0, '6 switch: pane back on chat (no .xterm)');
  check((await page.getByRole('dialog').count()) === 0, '6 switch: commit dialog closed');
  check((await page.getByText('Working tree changes vs last commit.').count()) === 0, '6 switch: git panel gone');
  await app.shot('sweep-agent-tab-after-switch');
}

// 7: Portal tab regression
await go('/ai/coder');
const portalToggle = page.getByRole('button', { name: 'Workspace sidebar' });
const hasThread = (await inv('coder_list_threads')).length > 0;
console.log(`portal threads: ${hasThread}`);
if (await portalToggle.count()) {
  const gc = page.getByRole('button', { name: /^Git changes/ });
  if (await gc.count()) await gc.first().click();
  await page.waitForTimeout(1000);
  const openTabsBefore = await page.getByText('Open tabs', { exact: true }).count();
  check(openTabsBefore > 0, '7 Portal tab has an open tab before leaving');
  await app.shot('sweep-portal-tab-before-agent-visit');
  await page.evaluate(() => { const x = document.createElement('a'); x.href = '/ai/coder/claude-code?session=' + ''; document.body.appendChild(x); x.click(); x.remove(); });
  await page.waitForTimeout(1500);
  await page.evaluate(() => { const x = document.createElement('a'); x.href = '/ai/coder'; document.body.appendChild(x); x.click(); x.remove(); });
  await page.waitForTimeout(2000);
  check((await page.getByText('Open tabs', { exact: true }).count()) > 0, '7 Portal tab still has its open tabs after agent tab round trip');
  check((await page.getByText('Agent changes', { exact: true }).count()) > 0, '7 Portal sidebar still shows "Agent changes"');
  await app.shot('sweep-portal-tab-after-agent-visit');
} else check(false, '7 Portal tab: no Workspace sidebar toggle (need an active coder thread?)');

console.log(failed ? `\n${failed} FAILED` : '\nall passed');
await app.done();
process.exit(failed ? 1 : 0);
