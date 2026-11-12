import { open } from './app.mjs';
import fs from 'node:fs'; import path from 'node:path'; import cp from 'node:child_process';
const app = await open('projects/14'); const p = app.page;
const all = await p.evaluate(async () => (await window.__TAURI_INTERNALS__.invoke('get_all_projects')).map(x=>({id:x.id,name:x.name,path:x.path})));
const ids=[228,12,22,13,15,19,10,4,14,23,61,18,66,41,31,231,176];
for (const id of ids){ const pr=all.find(x=>x.id==id); if(!pr){console.log(id,'missing');continue;}
 let line=`#${id} ${pr.name} ${pr.path} | `;
 try{ const pj=JSON.parse(fs.readFileSync(path.join(pr.path,'package.json'),'utf8')); line+='scripts: '+Object.keys(pj.scripts||{}).join(','); }catch{ line+='no package.json: '+fs.readdirSync(pr.path).filter(f=>/pom|gradle|Cargo|pubspec|pyproject|Makefile|requirements|go.mod|app.json/i.test(f)).join(','); }
 try{ line+=' | git: '+(cp.execSync('git status --porcelain',{cwd:pr.path}).toString().split('\n').filter(Boolean).length)+' changes'; }catch{line+=' | git:n/a'}
 console.log(line);}
await app.done();
