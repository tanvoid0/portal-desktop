import { open } from './app.mjs';
let app = await open('tasks/54'); let p = app.page;
const st = async () => (await p.locator('main').innerText()).match(/Status\n(.*)/)?.[1];
console.log('start', await st());
for (let i=0;i<4;i++){ await p.getByRole('button',{name:/Toggle Status/}).click(); await p.waitForTimeout(900); console.log('toggle ->', await st()); }
// edit page
await p.goto(p.url().replace(/\/tasks\/54.*/, '/tasks/54/edit')); await p.waitForTimeout(1200);
console.log('edit title:', await p.inputValue('#title'));
await p.fill('#title','QA Gamma edited'); await p.keyboard.press('Control+Enter'); await p.waitForTimeout(1500);
console.log('after ctrl+enter ->', p.url());
await app.shot('sweep-tasks-detail-after-edit');
// delete from detail: sub-task parent cascade
await p.goto(new URL('/tasks/52', p.url()).href); await p.waitForTimeout(1200);
await p.getByRole('button',{name:'Delete'}).first().click(); await p.waitForTimeout(600);
console.log('dialog:', await p.getByRole('alertdialog').or(p.getByRole('dialog')).innerText().catch(()=>'none'));
await app.shot('sweep-tasks-detail-delete-confirm');
await p.keyboard.press('Escape'); await p.waitForTimeout(400);
console.log('still there after esc:', p.url());
await app.done();
