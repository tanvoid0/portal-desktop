import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
console.log('healthy:', (await p.locator('main').innerText()).includes('Backlog'));
const cards=async()=>p.locator('[aria-label^="Select task: QA"]').evaluateAll(a=>a.map(x=>x.getAttribute('aria-label').replace('Select task: ','')));
const chips=async()=>{const t=await p.locator('main').innerText(); return t.split('Clear All')[0].split('Kanban')[1]?.replace(/\n+/g,' | ')};
const applyDlg=async(label,section)=>{await p.keyboard.press('Control+f'); await p.waitForTimeout(400); const d=p.getByRole('dialog'); await d.getByText(label,{exact:true}).first().click(); await p.getByRole('button',{name:/Apply/}).click(); await p.waitForTimeout(700)};
await applyDlg('To Do'); console.log('status To Do ->', await cards(), '| chips:', await chips()); await app.shot('sweep-tasks-chip-status');
await applyDlg('High'); console.log('+ High ->', await cards(), '| chips:', await chips());
await applyDlg('Bug'); console.log('+ Bug ->', await cards(), '| chips:', await chips()); await app.shot('sweep-tasks-chips-status-priority-type');
// remove the priority chip via its X
const chipX=p.locator('main').getByRole('button',{name:/remove|clear/i}); console.log('x buttons', await chipX.count(), await chipX.evaluateAll(a=>a.map(x=>x.getAttribute('aria-label'))));
if(await chipX.count()){ await chipX.nth(1).click(); await p.waitForTimeout(600); console.log('removed #2 ->', await cards(), '| chips:', await chips()); }
await p.getByRole('button',{name:'Clear All'}).first().click(); await p.waitForTimeout(600);
console.log('clear all ->', await cards(), '| chips:', await chips(), 'filter badge', await p.getByRole('button',{name:/Filters/}).innerText());
await app.done();
