import { open } from './app.mjs';

const app = await open('devices');
await app.page.waitForTimeout(500);

const info = await app.page.evaluate(() => {
  // @ts-ignore
  const internals = window.__TAURI_INTERNALS__;
  const desc = Object.getOwnPropertyDescriptor(internals, 'invoke');
  let assignWorked = 'n/a';
  try {
    const before = internals.invoke;
    internals.invoke = () => 'patched';
    assignWorked = internals.invoke === before ? 'no-op (unchanged)' : 'changed';
  } catch (e) {
    assignWorked = 'threw: ' + String(e);
  }
  return {
    hasInternals: !!internals,
    hasTauriCore: !!window.__TAURI__?.core,
    descriptor: desc
      ? { writable: desc.writable, configurable: desc.configurable, hasGetter: !!desc.get }
      : null,
    assignWorked,
  };
});
console.log(JSON.stringify(info, null, 2));

await app.done();
