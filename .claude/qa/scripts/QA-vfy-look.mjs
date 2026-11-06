import { open } from './app.mjs';
const app = await open('tasks/84'); const p = app.page;
console.log((await p.locator('main').innerText()).slice(0,500));
console.log(await p.getByRole('button').allInnerTexts());
await app.shot('sweep-task-detail'); await app.done();
