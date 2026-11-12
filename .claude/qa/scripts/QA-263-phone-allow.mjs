// Real phone pair: approve the newest pending request from the phone (by id, not card position —
// stale pending rows stack up on the Devices page).
import { open } from './app.mjs';
const app = await open('devices'); const p = app.page; await p.waitForTimeout(2000);
const inv = (c, a) => p.evaluate(({ c, a }) => window.__TAURI_INTERNALS__.invoke(c, a), { c, a });
const l = await inv('list_devices', {});
const row = l.filter((d) => /S26/.test(d.device_name) && d.approval_type === 'pending')
  .sort((a, b) => String(b.created_at).localeCompare(String(a.created_at)))[0];
console.log('newest pending:', row?.device_id, row?.created_at);
const caps = ['agents', 'coder', 'dashboard', 'projects', 'tasks', 'terminal', 'terminal.exec'];
console.log('approve:', JSON.stringify(await inv('approve_device', { request: { device_id: row.device_id, approval_type: 'until_revoked', capabilities: caps } })));
await app.done();
