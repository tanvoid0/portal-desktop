import { open } from './app.mjs';
const app = await open('tasks');
const p = app.page;
const r = await p.evaluate(async () => {
  const all = await window.__TAURI_INTERNALS__.invoke('get_all_projects');
  return { n: all.length, p1: all.filter(x => String(x.id) === '1' || /^Project/.test(x.name)).map(x=>({id:x.id,name:x.name,arch:x.archived||x.is_archived})), ids: all.slice(0,3).map(x=>x.id), keys: Object.keys(all[0]).join(',') };
});
console.log(JSON.stringify(r));
await app.done();
