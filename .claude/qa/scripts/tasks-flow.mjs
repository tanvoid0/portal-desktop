import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const app = await open('tasks/create');
const p = app.page;
await dismissVault(p);
const btns = await p.locator('main button').evaluateAll(els=>els.map(e=>e.textContent.trim()).filter(Boolean));
console.log(btns.slice(-8));
// empty submit
const submit = p.getByRole('button', { name: /^(Create Task|Create|Save)/ }).last();
await submit.click(); await p.waitForTimeout(600);
console.log('empty submit toast:', await p.evaluate(()=>[...document.querySelectorAll('[data-sonner-toast]')].map(e=>e.innerText)));
await p.getByPlaceholder('Enter task title...').fill('QA task 🚀 日本語 <b>x</b> ' + 'long '.repeat(40));
await p.getByPlaceholder('Enter task description...').fill('QA fixture\nline2');
console.log(await app.shot('sweep-tasks-create-filled'));
await submit.dblclick();
await p.waitForTimeout(1800);
console.log('url', p.url());
console.log(await app.shot('sweep-tasks-after-create'));
await app.done();
