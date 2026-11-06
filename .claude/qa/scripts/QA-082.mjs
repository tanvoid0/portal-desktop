import { open } from './app.mjs';
const app = await open('tasks?project=all&view=archive'); const p = app.page;
const txt=async()=>(await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,400);
console.log('archive now:', await txt());
await app.shot('QA-082-after-archive-current');
await p.getByPlaceholder('Search tasks...').fill('zzzz'); await p.waitForTimeout(600);
console.log('no match:', await txt()); await app.shot('QA-082-after-archive-nomatch');
const c=p.getByRole('button',{name:/Clear filters/i}); console.log('clear btn', await c.count());
if(await c.count()){await c.first().click(); await p.waitForTimeout(500); console.log('after clear:', await txt(), 'search=', await p.getByPlaceholder('Search tasks...').inputValue());}
await app.done();
