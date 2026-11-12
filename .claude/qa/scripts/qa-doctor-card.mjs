// usage: CDP_PORT=9223 node qa-doctor-card.mjs <projectId> <shot>  -- prints the failure card text
import { open } from './app.mjs';
const [id, shot] = process.argv.slice(2);
const app = await open(`projects/${id}/pipelines`);
const p = app.page;
await p.getByText('Local run history').first().waitFor({ timeout: 60000 });
await p.waitForTimeout(2000);
const card = p.locator('div').filter({ hasText: /^Run failed/ }).last();
console.log('card count', await p.getByText('Run failed').count());
const txt = await p.evaluate(() => { const e=[...document.querySelectorAll('*')].find(x=>x.children.length&&/^Run failed/.test(x.innerText||'')&&x.innerText.length<1500); return e? e.parentElement.innerText : null; });
console.log(txt);
await p.getByText('Run failed').first().scrollIntoViewIfNeeded();
await p.waitForTimeout(300);
console.log(await app.shot(shot));
await app.done();
