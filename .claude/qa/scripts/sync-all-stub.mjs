// Drives the Sync all sheet with a FAKE report: no real sync runs (IPC for sync_all_projects is
// answered in the page). Aborts unless the interception is proven first. Never sends the prompt.
import { open } from './app.mjs';

const step = (s, status, detail = null) => ({ step: s, status, detail });
const proj = (id, name, path, steps) => ({
  id,
  name,
  path,
  steps,
  ahead: 0,
  behind: 0,
  dirty: 0,
  untracked: 0,
});
const diverged = [
  step('fetch', 'ok'),
  step('pull', 'attention', 'diverged (2 ahead, 3 behind) — needs manual merge'),
  step('push', 'skipped', 'pull did not complete'),
  step('mark', 'attention', '4 modified, 1 untracked — commit manually'),
];
const failed = [
  step(
    'fetch',
    'failed',
    "fatal: unable to access 'https://example.invalid/a-very-long-organisation-name/a-very-long-repository-name-without-breaks.git/': Could not resolve host: example.invalid"
  ),
  step('pull', 'skipped', 'fetch failed'),
  step('push', 'skipped', 'fetch failed'),
  step('mark', 'ok'),
];

export const rows = (base = 'D:\\QA fixtures') => [
  proj(990001, 'QA diverged repo', `${base}\\one`, diverged),
  proj(
    990002,
    'QA_a_very_long_unbroken_project_name_that_keeps_going_and_going_and_going_forever_and_ever',
    `${base}\\deep\\nested folder\\two`,
    failed
  ),
  proj(990003, 'QA ünïcode 🚀 repo', `${base}\\three 🚀`, diverged),
];

const report = (projects) => ({
  projects,
  envNote: 'Vault locked — env files not backed up',
  snapshot: step('snapshot', 'skipped', 'no snapshot password'),
  cancelled: false,
  startedAt: new Date().toISOString(),
  finishedAt: new Date().toISOString(),
});

/** Reload /projects, install the stub, prove it, then go through the real start dialog. */
export async function fakeSync(page, projects, shotDialog) {
  await page.goto('http://tauri.localhost/projects');
  await page.waitForTimeout(2500);
  const proof = await page.evaluate(
    async (fake) => {
      const real = window.fetch.bind(window);
      const fakes = { qa_probe: 'intercepted', sync_all_projects: fake, cancel_sync_all: null };
      window.__qaHits = [];
      window.fetch = (input, init) => {
        const cmd = decodeURIComponent(
          String(input?.url ?? input).split('ipc.localhost/')[1] ?? ''
        );
        if (!(cmd in fakes)) return real(input, init);
        window.__qaHits.push(cmd);
        return Promise.resolve(
          new Response(JSON.stringify(fakes[cmd]), {
            headers: { 'Content-Type': 'application/json', 'Tauri-Response': 'ok' },
          })
        );
      };
      return window.__TAURI_INTERNALS__.invoke('qa_probe').catch(String);
    },
    report(projects)
  );
  if (proof !== 'intercepted') throw new Error(`stub not effective, NOT starting a sync: ${proof}`);

  await page.getByRole('button', { name: 'Sync all', exact: true }).click();
  await page.waitForTimeout(1500);
  const dialogs = page.locator('[role=dialog], [role=alertdialog]');
  const text = async () => (await dialogs.allInnerTexts()).join(' | ').replace(/\n+/g, ' / ');
  console.log('start dialog:', await text());
  if (/unlock/i.test((await text()).split('Continue?')[0]) && !/Continue\?/.test(await text())) {
    await dialogs.getByRole('button', { name: /cancel/i }).click();
    await page.waitForTimeout(1500);
    console.log('after unlock cancel:', await text());
  }
  if (shotDialog) await shotDialog();
  await dialogs.getByRole('button', { name: /^(sync|continue)$/i }).click();
  await page.getByText('Snapshot:').waitFor({ timeout: 5000 });
  console.log('stub hits:', await page.evaluate(() => window.__qaHits.join()));
}

export { open };
