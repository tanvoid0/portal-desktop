import { connect } from './cdp.mjs';
const app = await connect();
const inv = (c,a)=>app.eval((c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),c,a);
let ss; try { ss = await inv('agents_list_sessions'); } catch(e){ console.log('list err', String(e).slice(0,200)); ss=[]; }
console.log('sessions', ss.length, ss.slice(0,3).map(s=>[s.id,s.backend_id,s.cwd]));
const s = ss.filter(s=>s.backend_id==='claude-code'&&s.cwd)[0] ?? ss[0];
if (!s) { console.log('NO SESSION'); process.exit(1); }
await app.nav('/ai/coder/claude-code?session='+s.id, 3000);
for (const w of [1280,1440,1920]) {
  await app.metrics(w,800); await app.wait(800);
  await app.eval(()=>{ const b=[...document.querySelectorAll('button')].filter(b=>b.getAttribute('aria-label')?.includes('Workspace sidebar'))[0]; if(b && b.getAttribute('aria-pressed')==='false') b.click(); });
  await app.clickButton('Files');
  await app.wait(1000);
  const r = await app.eval(()=>{
    const c=[...document.querySelectorAll('textarea,input')].filter(e=>/message|reply/i.test(e.getAttribute('placeholder')||''))[0];
    const ta=c?.getBoundingClientRect();
    const tabs=[...document.querySelectorAll('[role=tab]')].map(t=>t.innerText.trim()+':'+Math.round(t.getBoundingClientRect().width));
    const asides=[...document.querySelectorAll('aside')].map(a=>Math.round(a.getBoundingClientRect().width));
    let el=c, chat=null; while(el){ if(el.className?.toString?.().includes('min-w-80')){chat=el;break;} el=el.parentElement; }
    return {composerW: ta&&Math.round(ta.width), chatW: chat&&Math.round(chat.getBoundingClientRect().width), tabs, asides, sessionsBtn: [...document.querySelectorAll('button')].some(b=>/Sessions/.test(b.innerText))};
  });
  console.log(w, JSON.stringify(r));
  console.log(await app.shot(`QA-159-after-agent-files-${w}`));
}
await app.resetMetrics(); await app.close(); process.exit(0);
