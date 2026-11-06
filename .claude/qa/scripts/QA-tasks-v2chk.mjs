import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
console.log('date in page', await p.evaluate(()=>new Date().toISOString()), 'real', new Date().toISOString());
await app.done();
