// Sniff the actual request URL Tauri's postMessage/fetch IPC fallback uses,
// so we can target agents_list_backends specifically with page.route().
import { open } from './app.mjs';

const app = await open('agents/run');
app.page.on('request', (req) => {
  const url = req.url();
  if (/ipc|localhost/i.test(url) && !url.startsWith('http://localhost:1420')) {
    console.log('REQ', req.method(), url);
  }
});
await app.page.reload();
await app.page.waitForTimeout(2000);
await app.done();
