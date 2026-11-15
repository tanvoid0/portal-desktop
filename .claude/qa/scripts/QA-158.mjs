// Repro: Action Runs page: selecting a run sets ?run= but no detail pane appears
import { open } from './app.mjs';
const app = await open('pipeline-runs');
const { page } = app;
await page.waitForTimeout(3000);
await page.getByRole('button', { name: /QA Actions Fixture/ }).first().click();
await app.shot('QA-158-before-no-detail-pane');
await page.waitForTimeout(2000);
console.log('url', page.url());
console.log('Run details in DOM:', await page.getByText('Run details').count(), 'Run not found:', await page.getByText('Run not found').count(), 'asides', await page.locator('aside, [data-slot=sheet-content]').count());
const id = new URL(page.url()).searchParams.get('run');
await page.goto(new URL('/pipeline-runs?run=' + id, page.url()).href);
await page.waitForTimeout(3000);
console.log('direct load: Run details in DOM:', await page.getByText('Run details').count());
await app.shot('QA-158-before-direct-load');
await app.done();
