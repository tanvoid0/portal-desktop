// raw CDP attach, NO navigation: node qa-raw.mjs <shotName> [selectorText]
import Module from 'node:module';
const o = Module.prototype._compile; Module.prototype._compile = function(c,f){ if(f.endsWith('coreBundle.js')) c=c.replace('assert(targetInfo.browserContextId, "targetInfo: " + JSON.stringify(targetInfo, null, 2));',''); return o.call(this,c,f); };
const { chromium } = await import('@playwright/test');
const b = await chromium.connectOverCDP('http://127.0.0.1:9223'); const p = b.contexts()[0].pages()[0];
const [name, text] = process.argv.slice(2);
console.log(p.url());
if (text) await p.getByText(text).first().scrollIntoViewIfNeeded();
await p.waitForTimeout(400);
await p.screenshot({ path: `D:/projects/production/portal/portal_desktop/.claude/qa/shots/${name}.png` });
console.log((await p.locator('body').innerText()).split('Generate a pipeline')[1]?.slice(0,700));
await b.close();
