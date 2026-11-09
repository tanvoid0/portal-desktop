import { open } from './app.mjs';

const expected = [
  ['View Pods', '/cloud/pods'],
  ['View Services', '/cloud/services'],
  ['View Deployments', '/cloud/deployments'],
  ['View StatefulSets', '/cloud/statefulsets'],
  ['View DaemonSets', '/cloud/daemonsets'],
  ['View Jobs', '/cloud/jobs'],
  ['View CronJobs', '/cloud/cronjobs'],
  ['View ConfigMaps', '/cloud/configmaps'],
  ['View Secrets', '/cloud/secrets'],
  ['View Ingress', '/cloud/ingress'],
];

for (const [label, expectedPath] of expected) {
  const app = await open('cloud/workloads');
  await app.page.waitForTimeout(1200);
  const tile = app.page.getByRole('button', { name: new RegExp(label, 'i') });
  const count = await tile.count();
  if (count === 0) {
    console.log(`${label}: TILE NOT FOUND`);
    await app.done();
    continue;
  }
  await tile.first().click();
  await app.page.waitForTimeout(800);
  const url = new URL(app.page.url()).pathname;
  console.log(`${label}: navigated to ${url} (expected ${expectedPath}) -> ${url === expectedPath ? 'OK' : 'MISMATCH'}`);
  await app.done();
}
