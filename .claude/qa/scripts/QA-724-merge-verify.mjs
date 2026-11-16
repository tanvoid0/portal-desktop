// #724: live-verify the Agents→Coder merge (#720) on a dev instance over CDP.
//   node .claude/qa/scripts/QA-724-merge-verify.mjs        → checks 1, 3, 5 + tab restore + widths
//   node .claude/qa/scripts/QA-724-merge-verify.mjs run    → checks 2, 4 (starts a real Claude Code run)
// Prints one PASS/FAIL line per assertion.
import { open } from './app.mjs';

const SCRATCH = process.env.QA_SCRATCH ?? process.env.TEMP;
const app = await open('ai/coder');
const { page } = app;
const inv = (cmd, args) =>
  page.evaluate(([c, a]) => window.__TAURI_INTERNALS__.invoke(c, a), [cmd, args]);
const results = [];
const check = (ok, msg) => {
  results.push(ok);
  console.log(`${ok ? 'PASS' : 'FAIL'} ${msg}`);
};
const url = () => new URL(page.url());
const rel = () => url().pathname + url().search;
const go = async (route) => {
  await page.goto(new URL(route, page.url()).href);
  await page.waitForLoadState('load');
  await page.waitForTimeout(1500);
};
const settle = (ms = 1200) => page.waitForTimeout(ms);
// The launcher's cwd is a ProjectCombobox: the value shows in its trigger button.
const hasCwd = (v) =>
  page.locator('button').evaluateAll((els, v) => !!v && els.some((e) => e.innerText.includes(v)), v);
const engine = page.locator('[aria-label="Coding engine"]');
const engineTab = (name) => engine.locator('[role="tab"]').filter({ hasText: name });
// Client-side nav (no webview reload) — like a sidebar link click.
const clientGoto = (route) =>
  page.evaluate((r) => {
    const a = document.createElement('a');
    a.href = r;
    document.body.appendChild(a);
    a.click();
    a.remove();
  }, route);

if (process.argv[2] !== 'run') {
  // (1) engine strip on /ai/coder, Portal active, both directions work
  await engine.locator('[role="tab"]').first().waitFor({ timeout: 10000 }).catch(() => {});
  const names = await engine.locator('[role="tab"]').allInnerTexts();
  check(names.join('|') === 'Portal|Claude Code', `(1) engine tabs = ${names.join('|')}`);
  const active = await engine.locator('[role="tab"][data-state="active"]').allInnerTexts();
  check(active.join() === 'Portal', `(1) active on /ai/coder = ${active.join()}`);
  const top = await page.locator('[role="tablist"]').first().locator('[role="tab"]').allInnerTexts();
  check(!top.some((t) => /agents/i.test(t)), `(1) top tabs = ${top.join('|')} (no Agents)`);
  await engineTab('Claude Code').click();
  await settle();
  check(url().pathname === '/ai/coder/claude-code', `(1) click Claude Code → ${rel()}`);
  await engineTab('Portal').click();
  await settle();
  check(url().pathname === '/ai/coder', `(1) click Portal → ${rel()}`);

  // (1) Portal ?id= deep link still selects that thread
  const threads = await inv('coder_list_threads');
  const t = threads[1] ?? threads[0];
  if (t) {
    await go(`/ai/coder?id=${t.id}`);
    const body = await page.locator('main').innerText();
    const title = (t.title ?? '').slice(0, 24);
    check(
      url().searchParams.get('id') === t.id && (!title || body.includes(title)),
      `(1) ?id deep link keeps thread "${title}"`
    );
  } else check(false, '(1) no coder threads to deep-link');

  // (3) old routes redirect to the Claude Code tab with focus/prefill
  const sessions = (await inv('agents_list_sessions')).filter((s) => s.backend_id === 'claude-code');
  await go('/ai/agents');
  check(url().pathname === '/ai/coder/claude-code', `(3) /ai/agents → ${rel()}`);
  await go(`/ai/agents/run?cwd=${encodeURIComponent(SCRATCH)}`);
  const cwdOk = await hasCwd(SCRATCH);
  check(url().pathname === '/ai/coder/claude-code' && cwdOk, `(3) /ai/agents/run?cwd= → ${url().pathname}, cwd prefilled=${cwdOk}`);
  const s = sessions[0];
  if (s) {
    await go(`/ai/agents/sessions/${s.id}`);
    check(
      url().pathname === '/ai/coder/claude-code' && url().searchParams.get('session') === s.id,
      `(3) /ai/agents/sessions/<id> → ${rel()}`
    );
    // #722 review fix: tab round-trip keeps the focused session (deep-link entry)
    await engineTab('Portal').click();
    await settle();
    await engineTab('Claude Code').click();
    await settle();
    check(url().searchParams.get('session') === s.id, `(722) Portal→Claude Code restores session → ${rel()}`);
    // and after a client-side entry + sidebar pick of another session
    const s2 = sessions[1];
    if (s2) {
      await clientGoto(`/ai/coder/claude-code?session=${s2.id}`);
      await settle();
      await engineTab('Portal').click();
      await settle();
      await engineTab('Claude Code').click();
      await settle();
      check(url().searchParams.get('session') === s2.id, `(722) restore after client-side entry → ${rel()}`);
    }
  } else check(false, '(3) no claude-code sessions to deep-link');

  // (5) "Run agent here" from a project row
  await go('/projects');
  const runBtn = page.locator('button[aria-label^="Run agent on "]:visible').first();
  const label = await runBtn.getAttribute('aria-label').catch(() => null);
  if (label) {
    const projects = await inv('get_all_projects');
    const proj = projects.find((p) => `Run agent on ${p.name}` === label);
    await runBtn.hover().catch(() => {});
    await runBtn.click({ force: true });
    await settle(2000);
    const cwd = await hasCwd(proj?.path);
    check(url().pathname === '/ai/coder/claude-code' && cwd, `(5) project "${proj?.name}" → ${rel()}, cwd prefilled=${cwd}`);
  } else check(false, '(5) no "Run agent on" button on /projects');

  // (5) "Run agent here" from a task row's context menu (needs a task with a project path)
  await go('/tasks');
  const rows = page.locator('[data-slot="context-menu-trigger"]:visible');
  let launched = false;
  for (let i = 0; i < Math.min(await rows.count(), 15) && !launched; i++) {
    await rows.nth(i).click({ button: 'right' });
    const item = page.locator('[role="menu"]').getByText('Run agent here', { exact: true });
    if (await item.isVisible().catch(() => false)) {
      await item.click();
      launched = true;
    } else await page.keyboard.press('Escape');
  }
  if (launched) {
    await settle(2000);
    const prompt = await page.locator('#agent-prompt').inputValue().catch(() => '');
    check(
      url().pathname === '/ai/coder/claude-code' && prompt.length > 0,
      `(5) task → ${rel()}, prompt prefilled=${prompt.length > 0}`
    );
  } else check(false, '(5) no task row offered "Run agent here"');

  // widths: strip scrolls, never clips; no new row
  const cdp = await page.context().newCDPSession(page);
  for (const [w, h] of [[1024, 600], [760, 600], [2560, 1080]]) {
    await cdp.send('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
    await go('/ai/coder/claude-code');
    const m = await engine.evaluate((el) => {
      const r = el.getBoundingClientRect();
      const bar = document.querySelector('[role="tablist"]').getBoundingClientRect();
      const cs = getComputedStyle(el);
      return { top: Math.round(r.top), barTop: Math.round(bar.top), sw: el.scrollWidth, cw: el.clientWidth, ox: cs.overflowX, right: Math.round(r.right), vw: innerWidth };
    }).catch((e) => ({ err: String(e) }));
    const ok = !m.err && Math.abs(m.top - m.barTop) < 12 && (m.sw <= m.cw || m.ox === 'auto') && m.right <= m.vw;
    check(ok, `width ${w}x${h}: ${JSON.stringify(m)}`);
    await app.shot(`QA-724-engine-tabs-${w}x${h}`);
  }
  await cdp.send('Emulation.clearDeviceMetricsOverride');
} else {
  // (2) start a run, hit an approval, reload → session + approval come back
  await go(`/ai/coder/claude-code?cwd=${encodeURIComponent(SCRATCH)}`);
  await page.getByLabel('First message').fill(
    'Use the Write tool to create a file named qa724.txt with content "hi" in the current directory. Then stop.'
  );
  await page.locator('button').filter({ hasText: 'Start run' }).click();
  // (4) leave for another page before the approval arrives — client-side, no reload
  await settle(800);
  const sessionUrl = rel();
  await clientGoto('/projects');
  // App's own toast store (notify.ts → toastActions), not sonner.
  await page.getByText('Agent needs your approval').first().waitFor({ timeout: 90000 }).catch(() => {});
  const toastOpen = page.locator('button').filter({ hasText: /^Open$/ });
  const stackCount = await page.locator('.fixed.right-4.bottom-4').getByText('Approval required').count();
  check(stackCount === 1, `(4) floating stack on /projects shows ${stackCount} approval(s)`);
  await app.shot('QA-724-stack-off-page');
  const hadToast = await toastOpen.first().isVisible().catch(() => false);
  check(hadToast, '(4) approval toast with Open action on /projects');
  if (hadToast) await toastOpen.first().click();
  await settle(2000);
  const sid = url().searchParams.get('session');
  check(url().pathname === '/ai/coder/claude-code' && !!sid, `(4) toast Open → ${rel()} (launched from ${sessionUrl})`);
  const focusedBanners = await page.getByText('Approval required').count();
  check(focusedBanners === 1, `(4) focused session shows ${focusedBanners} approval banner(s) (stack must not duplicate)`);
  await app.shot('QA-724-focused-no-dup');

  await go(rel());
  const after = await page.getByText('Approval required').count();
  check(after >= 1 && url().searchParams.get('session') === sid, `(2) after reload: banner=${after}, ${rel()}`);
  await app.shot('QA-724-after-reload');

  for (let i = 0; i < 5; i++) {
    const accept = page.locator('button').filter({ hasText: 'Accept once' }).first();
    if (!(await accept.isVisible().catch(() => false))) break;
    await accept.click();
    await settle(1500);
  }
  check(!(await page.getByText('Approval required').count()), '(2) approval decided, banner gone');
  await page.locator('button').filter({ hasText: 'End run' }).click().catch(() => {});
}

console.log(`\n${results.filter(Boolean).length}/${results.length} passed`);
await app.done();
