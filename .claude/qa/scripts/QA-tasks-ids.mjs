import { open } from './app.mjs';
const app = await open('tasks?project=all&view=list'); const p = app.page;
for (const t of ['QA Alpha','QA Beta','QA Gamma','QA Cancelme','QA Long','QA Nospace']) {
  await p.goto(new URL('/tasks?project=all&view=list', p.url()).href); await p.waitForTimeout(900);
  const l = p.getByText(new RegExp('^'+t)).first(); if(!await l.count()){console.log(t,'none');continue;}
  await l.click(); await p.waitForTimeout(700); console.log(t, p.url());
}
await app.done();
