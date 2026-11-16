// #690 + #769 live check on a dev instance over CDP (node scripts/dev-instance.mjs 1).
//   node .claude/qa/scripts/QA-690-769-verify.mjs       → #690 only
//   node .claude/qa/scripts/QA-690-769-verify.mjs run   → also #769 (starts a real Claude Code run)
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
const go = async (route) => {
  await page.goto(new URL(route, page.url()).href);
  await page.waitForLoadState('load');
  await page.waitForTimeout(1500);
};
// Client-side nav (no remount of an already-mounted page) — like a notification goto.
const clientGoto = (route) =>
  page.evaluate((r) => {
    const a = document.createElement('a');
    a.href = r;
    document.body.appendChild(a);
    a.click();
    a.remove();
  }, route);
const activeCard = () =>
  page.locator('[role="group"].bg-primary').first().getAttribute('aria-label').catch(() => null);

// #690: already on /ai/coder?id=A, goto ?id=B → B becomes the active thread.
const threads = (await inv('coder_list_threads')).filter((t) => t.title);
const [a, b] = threads;
if (!a || !b || a.title === b.title) {
  check(false, `#690 needs 2 coder threads with distinct titles (have ${threads.length})`);
} else {
  await go(`/ai/coder?id=${a.id}`);
  check((await activeCard()) === a.title, `#690 deep link → active "${await activeCard()}" (want "${a.title}")`);
  await clientGoto(`/ai/coder?id=${b.id}`);
  await page.waitForTimeout(1500);
  check((await activeCard()) === b.title, `#690 client goto → active "${await activeCard()}" (want "${b.title}")`);
  await clientGoto('/ai/coder');
  await page.waitForTimeout(1000);
  check((await activeCard()) === b.title, `#690 bare tab link keeps "${await activeCard()}"`);
}

if (process.argv[2] === 'run') {
  // #769: run awaiting approval, reload on /projects → floating stack still shows it.
  await go(`/ai/coder/claude-code?cwd=${encodeURIComponent(SCRATCH)}`);
  await page.getByLabel('First message').fill(
    'Use the Write tool to create a file named qa769.txt with content "hi" in the current directory. Then stop.'
  );
  await page.locator('button').filter({ hasText: 'Start run' }).click();
  await page.waitForTimeout(800);
  await clientGoto('/projects');
  const stack = () => page.locator('.fixed.right-4.bottom-4').getByText('Approval required');
  await stack().first().waitFor({ timeout: 90000 }).catch(() => {});
  check((await stack().count()) === 1, `#769 stack before reload: ${await stack().count()}`);
  await page.reload();
  await page.waitForLoadState('load');
  await stack().first().waitFor({ timeout: 15000 }).catch(() => {});
  check((await stack().count()) === 1, `#769 stack after reload on /projects: ${await stack().count()}`);
  await app.shot('QA-769-stack-after-reload');

  // cleanup: end every run this started
  for (const sid of await inv('agents_list_running')) {
    await go(`/ai/coder/claude-code?session=${sid}`);
    await page.locator('button').filter({ hasText: 'End run' }).click().catch(() => {});
  }
}

console.log(`\n${results.filter(Boolean).length}/${results.length} passed`);
await app.done();
