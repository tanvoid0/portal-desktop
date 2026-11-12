import { open } from './app.mjs';
const app = await open('ai/coder'); const p = app.page; await p.waitForTimeout(2500);
const r = await p.evaluate(async () => { const l = await window.__TAURI_INTERNALS__.invoke('coder_list_threads'); return l.filter(t=>/A Portal Action command failed|Create or extend this project/.test(t.title||'')).map(t=>({id:t.id,title:t.title.slice(0,50),prov:t.llm_provider,model:t.llm_model,status:t.status,ws:t.workspace_path||t.workspace||t.cwd,created:t.created_at,keys:Object.keys(t).join(',')})); });
console.log(JSON.stringify(r,null,1)); await app.done();
