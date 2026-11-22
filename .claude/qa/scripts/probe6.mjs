import { open } from './app.mjs';
const app = await open('tasks');
const r = await app.page.evaluate(async () => {
  const all = await window.__TAURI_INTERNALS__.invoke('get_all_projects');
  return all.filter(x => /^zz-|QA/.test(x.name)).map(x=>({id:x.id,name:x.name,path:x.path}));
});
console.log(JSON.stringify(r));
await app.done();
