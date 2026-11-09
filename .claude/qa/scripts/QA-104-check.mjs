// QA-104 verification: vault setup dialog should not block routes unless triggered.
import { open } from './app.mjs';

const routes = ['', 'projects', 'projects/audit', 'pipeline-runs', 'ai/coder', 'tasks'];
for (const r of routes) {
  const app = await open(r);
  await app.page.waitForTimeout(1500);
  const overlay = await app.page.locator('[data-dialog-overlay]').count();
  const vaultText = await app.page.getByText('Set up your vault').count();
  console.log(`route="${r || '/'}" overlay=${overlay} vaultDialogText=${vaultText}`);
  await app.done();
}
