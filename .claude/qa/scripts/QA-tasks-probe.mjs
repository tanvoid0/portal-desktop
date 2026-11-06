import { open } from './app.mjs';
const app = await open('tasks');
await app.shot('sweep-tasks-board-rewrite-wip');
console.log((await app.page.locator('main').innerText()).slice(0,500));
await app.done();
