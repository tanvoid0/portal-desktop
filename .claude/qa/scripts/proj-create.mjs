import { open } from './app.mjs';
import { dismissVault } from './util.mjs';
const FIX = 'C:/Users/tanvo/AppData/Local/Temp/claude/3de8d6c45086b260/26ec1573-83a5-4dbf-80b4-a05651f4e09c/scratchpad/qa-proj-fixture';
const app = await open('projects/create');
const p = app.page;
await dismissVault(p);
// 1) nonexistent path
await p.getByPlaceholder('/path/to/your/project').fill('Z:/does/not/exist/QA nope');
await p.waitForTimeout(600);
console.log('name auto-filled:', await p.getByPlaceholder('My Awesome Project').inputValue());
await p.getByPlaceholder('My Awesome Project').fill('QA project nonexistent');
await p.getByRole('button', { name: 'Create Project' }).click();
await p.waitForTimeout(1500);
console.log('nonexistent -> url', p.url(), 'toasts', await p.evaluate(()=>[...document.querySelectorAll('[data-sonner-toast]')].map(e=>e.innerText)));
console.log(await app.shot('sweep-projects-create-nonexistent-path'));
await app.done();
