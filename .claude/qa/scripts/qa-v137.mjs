import { open } from './app.mjs';
const app = await open('projects/14/pipelines/new'); const p = app.page; await p.waitForTimeout(2500);
console.log('new ->', p.url(), '|', (await p.locator('body').innerText()).replace(/\n+/g,' ').slice(0,200));
await app.shot('QA-137-after-pipelines-new');
await p.goto(new URL('/projects/14/pipelines', p.url()).href); await p.waitForTimeout(3000);
console.log('pipelines ->', p.url());
await p.goto(new URL('/pipeline-runs', p.url()).href); await p.waitForTimeout(2500);
console.log('pipeline-runs ->', p.url(), (await p.locator('body').innerText()).replace(/\n+/g,' ').slice(0,200));
await app.shot('QA-137-after-pipeline-runs');
for (const r of ['automation','automation/blocks','blocks']) { await p.goto(new URL('/'+r, p.url()).href); await p.waitForTimeout(2000); const t=await p.locator('body').innerText(); console.log(r, p.url(), 'Templates tab:', /\bTemplates\b/.test(t)); }
console.log(await app.shot('QA-137-after-blocks-page')); await app.done();
