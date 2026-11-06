// Repro: opening a run from the global Action Runs list, then "Back to Actions" / "Close" lands on the project's page, not the list.
import { open } from './app.mjs';
const app = await open('pipeline-runs');
const p = app.page;
for (const name of ['Back to Action Runs', 'Close']) {
  await p.goto(new URL('/pipeline-runs', p.url()).href);
  await p.waitForTimeout(1500);
  await p.getByText('Finalize test').first().click();
  await p.waitForTimeout(1000);
  await p.getByRole('button', { name, exact: true }).click();
  await p.waitForTimeout(1500);
  console.log(`${name} ->`, p.url(), '(expected /pipeline-runs)');
}
await app.shot('QA-042-before-back-to-actions-lands-on-project');
await app.done();
