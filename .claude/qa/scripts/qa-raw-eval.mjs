import Module from 'node:module';
const o = Module.prototype._compile; Module.prototype._compile = function(c,f){ if(f.endsWith('coreBundle.js')) c=c.replace('assert(targetInfo.browserContextId, "targetInfo: " + JSON.stringify(targetInfo, null, 2));',''); return o.call(this,c,f); };
const { chromium } = await import('@playwright/test');
const b = await chromium.connectOverCDP('http://127.0.0.1:9223'); const p = b.contexts()[0].pages()[0];
const fn = new Function('p', 'return (async()=>{' + process.argv[2] + '})()');
console.log(await fn(p)); await b.close();
