import Module from 'node:module';
const o = Module.prototype._compile; Module.prototype._compile = function(c,f){ if(f.endsWith('coreBundle.js')) c=c.replace('assert(targetInfo.browserContextId, "targetInfo: " + JSON.stringify(targetInfo, null, 2));',''); return o.call(this,c,f); };
const { chromium } = await import('@playwright/test');
const b = await chromium.connectOverCDP('http://127.0.0.1:9223'); const p = b.contexts()[0].pages()[0];
const [title, shot] = process.argv.slice(2);
const h = await p.evaluateHandle((title) => { for (const btn of document.querySelectorAll('button')) { if (btn.innerText.trim() !== 'Run') continue; let a = btn.parentElement; while (a && !(/^\s*/.test(a.innerText) && a.innerText.trimStart().startsWith(title) && [...a.querySelectorAll('button')].filter(x=>x.innerText.trim()==='Run').length===1)) a = a.parentElement; if (a) return btn; } return null; }, title);
const btn = h.asElement(); if(!btn) throw new Error('no btn'); await btn.click();
let last=''; const t0=Date.now();
while (Date.now()-t0<240000){ await p.waitForTimeout(4000); const t=await p.locator('body').innerText(); const i=t.indexOf('Local run history'); const s=t.slice(i,i+520).replace(/\n+/g,' | '); if(s!==last){console.log(`[${Math.round((Date.now()-t0)/1000)}s]`,s); last=s;} if(!/running/.test(s)&&Date.now()-t0>8000) break; }
await p.screenshot({path:`D:/projects/production/portal/portal_desktop/.claude/qa/shots/${shot}.png`});
await b.close();
