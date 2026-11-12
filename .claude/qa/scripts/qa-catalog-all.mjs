import { open } from './app.mjs';
const ids = process.argv.slice(2);
const app = await open('projects/14'); const p = app.page;
for (const id of ids) {
  await p.goto(`http://tauri.localhost/projects/${id}?tab=actions`); 
  await p.getByText('Local run history').first().waitFor({ timeout: 60000 }).catch(()=>console.log(id,'no history header'));
  await p.waitForTimeout(3500);
  const info = await p.evaluate(() => { const t=document.body.innerText; const path=(t.match(/[A-Z]:\[^\n]+/)||[])[0]; const cmds=[...document.querySelectorAll('button')].filter(b=>b.innerText.trim()==='Run').map(b=>{let a=b.parentElement; for(let i=0;i<4&&a;i++,a=a.parentElement){ if(a.innerText.length>12 && a.innerText.split('\n').length>=2) break;} return a.innerText.replace(/\n+/g,' | ').replace(/\| Run/,'').slice(0,110);}); return {name:document.title, path, cmds, head:t.slice(t.indexOf('Actions\n')).slice(0,150).replace(/\n+/g,' | ')}; });
  console.log('=== project', id, info.path); console.log(info.cmds.join('\n'));
}
await app.done();
