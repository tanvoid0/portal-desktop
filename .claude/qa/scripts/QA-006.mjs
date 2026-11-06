// QA-006: flipping the OS colour scheme while theme = "system" must not freeze the app.
// Raw CDP on purpose — Playwright's attach itself triggers the bug.
//   node .claude/qa/scripts/QA-006.mjs     (real app with CDP on :9222)
const list = await (await fetch('http://127.0.0.1:9222/json/list')).json();
const ws = new WebSocket(list.find((t) => t.type === 'page').webSocketDebuggerUrl);
let id = 0;
const pending = new Map();
const send = (method, params = {}) =>
  new Promise((res) => {
    const i = ++id;
    pending.set(i, res);
    ws.send(JSON.stringify({ id: i, method, params }));
  });
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && pending.has(d.id)) pending.get(d.id)(d);
};
await new Promise((r) => (ws.onopen = r));

const htmlClass = () =>
  Promise.race([
    send('Runtime.evaluate', { expression: 'document.documentElement.className' }).then(
      (d) => d.result?.result?.value ?? '?'
    ),
    new Promise((r) => setTimeout(() => r(null), 5000)),
  ]);
const flip = (value) =>
  send('Emulation.setEmulatedMedia', { media: '', features: [{ name: 'prefers-color-scheme', value }] });

console.log('theme in localStorage must be "system" (default) for this repro');
let ok = true;
for (const scheme of ['light', 'dark', 'light']) {
  await flip(scheme);
  await new Promise((r) => setTimeout(r, 1500));
  const cls = await htmlClass();
  const good = cls !== null && cls.split(' ').includes(scheme);
  console.log(`${scheme}: html.class=${cls ?? 'NO REPLY (frozen)'} ${good ? 'ok' : 'FAIL'}`);
  ok &&= good;
  if (cls === null) break;
}
await send('Emulation.setEmulatedMedia', { media: '', features: [] }); // back to the real OS value
console.log(ok ? 'PASS' : 'FAIL');
process.exit(ok ? 0 : 1);
