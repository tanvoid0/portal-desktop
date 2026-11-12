import { open } from './app.mjs';
const app = await open('ai/coder'); const p = app.page; await p.waitForTimeout(2500);
const r = await p.evaluate(async (ids) => { const l = await window.__TAURI_INTERNALS__.invoke('coder_list_threads'); return l.filter(t=>ids.includes(t.id)).map(t=>({id:t.id.slice(0,8),n:t.messages?.length,upd:t.updated_at,ws:t.workspace_root,pid:t.project_id,msgs:(t.messages||[]).map(m=>`${m.role}: ${String(m.content).slice(0,1500)}`)})); }, process.argv.slice(2));
console.log(JSON.stringify(r,null,1)); await app.done();
