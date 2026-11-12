import { open } from './app.mjs';
const app = await open('devices'); const p = app.page; await p.waitForTimeout(1500);
const inv = (c, a) => p.evaluate(({ c, a }) => window.__TAURI_INTERNALS__.invoke(c, a), { c, a });
const list = await inv('list_devices', {});
for (const d of list.filter((x) => /^QA263/.test(x.device_name))) {
  console.log(d.device_name, d.approval_type);
  await inv(d.approval_type === 'pending' ? 'reject_device' : 'revoke_device', { deviceId: d.device_id }).catch((e) => console.log('err', String(e)));
}
console.log('left:', (await inv('list_devices', {})).filter((x) => /^QA263/.test(x.device_name)).map((x) => x.approval_type));
await app.done();
