import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
await p.setViewportSize({width:1024,height:600}); await p.waitForTimeout(600);
await app.shot('sweep-tasks-board-1024x600');
await app.done();
