// Every route: h1 count, first h1 text, nested <main> count, document.title
import { open } from './app.mjs';
const app = await open(''); const p = app.page;
if (!app.mode.startsWith('tauri')) { console.log('NOT TAURI, stop'); process.exit(1); }
const routes = ['', 'projects', 'tasks', 'automation/scripts', 'terminal', 'ai', 'coder', 'cloud', 'credentials', 'deployments', 'documents', 'github', 'logs', 'pipeline-runs', 'sdk', 'settings', 'utilities'];
for (const r of routes) {
  await p.goto('http://localhost:1420/' + r); await p.waitForTimeout(2200);
  const i = await p.evaluate(() => ({ title: document.title, h1: [...document.querySelectorAll('h1')].map(h => h.innerText.trim()), mains: document.querySelectorAll('main').length }));
  console.log((r || '/').padEnd(20), 'title=' + JSON.stringify(i.title).padEnd(36), 'h1=' + JSON.stringify(i.h1), 'main=' + i.mains);
}
await p.goto('http://localhost:1420/');
await app.done();
