import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
for (const id of [84,85,86]) console.log(id, await p.evaluate(async(id)=>{try{const t=await window.__TAURI_INTERNALS__.invoke('update_task',{id,command:{status:'cancelled'}});return t.title+' '+t.status}catch(e){return 'ERR '+JSON.stringify(e)}},id));
await p.goto(new URL('/tasks?project=all&view=archive', p.url()).href); await p.waitForTimeout(1500);
const txt=async()=>(await p.locator('main').innerText()).replace(/\n+/g,' | ').slice(0,500);
const s=p.getByPlaceholder('Search tasks...');
console.log('archive:', await txt()); await app.shot('QA-082-after-archive-nonempty');
await s.fill('zzzz'); await p.waitForTimeout(700);
console.log('nomatch:', await txt()); await app.shot('QA-082-after-archive-nonempty-nomatch');
const c=p.getByRole('button',{name:/Clear filters/i}); console.log('clear btn', await c.count());
if(await c.count()){await c.first().click(); await p.waitForTimeout(600); console.log('after clear:', await txt(), '| search=', JSON.stringify(await s.inputValue()));}
await s.fill('Bravo'); await p.waitForTimeout(700); console.log('match Bravo:', await txt()); await app.shot('QA-082-after-archive-nonempty-match');
await s.fill(''); await app.done();
