import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
const get = (id)=>p.evaluate(async id=>{const t=await window.__TAURI_INTERNALS__.invoke('get_task',{id});return [t.status,t.completed_at]},id);
const col=(n)=>p.getByRole('region',{name:`Drop zone for ${n} tasks`});
// Beta -> Done, then clock +8d
await p.getByRole('button',{name:'Select task: QA Beta'}).dragTo(col('Done')).catch(e=>console.log('drag',e.message.slice(0,80))); await p.waitForTimeout(1000);
console.log('Beta', await get(68));
await p.clock.install({time:new Date(Date.now()+8*86400e3)}); await p.reload(); await p.waitForTimeout(1500);
console.log('footer text', await p.locator('main').evaluate(m=>[...m.querySelectorAll('*')].filter(e=>e.children.length==0&&/older in Archive/.test(e.textContent)).map(e=>e.textContent.trim())));
await app.shot('sweep-tasks-v2-older-footer');
const link = p.getByText(/older in Archive/).first();
if (await link.count()) { await link.click(); await p.waitForTimeout(800); console.log('link ->', p.url()); }
const row = p.locator('main').getByText('QA Beta',{exact:true}).locator('xpath=ancestor::div[.//button[contains(.,"Restore")]][1]');
await row.getByRole('button',{name:/Restore/}).click(); await p.waitForTimeout(300);
await app.shot('sweep-tasks-v2-restore-toast');
console.log('toasts', await p.locator('li[data-sonner-toast], [data-sonner-toast], [role=status]').allInnerTexts());
await p.waitForTimeout(1500);
console.log('Beta after restore', await get(68));
await p.goto(new URL('/tasks?project=all&view=board',p.url()).href); await p.waitForTimeout(1200);
console.log('Beta in Backlog', await col('Backlog').getByText('QA Beta').count(),'in Done', await col('Done').getByText('QA Beta').count());
await app.done();
