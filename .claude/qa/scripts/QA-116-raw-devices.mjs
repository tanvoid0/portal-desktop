import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

const raw = await app.page.evaluate(async () => {
  // @ts-ignore
  const invoke = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;
  return await invoke('list_devices');
});
console.log(JSON.stringify(raw, null, 2));

await app.done();
