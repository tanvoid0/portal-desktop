import { open } from './app.mjs';
const routes = process.argv.slice(2);
for (const r of routes) {
  const app = await open(r);
  const out=[];
  for (let i=0;i<14;i++){
    const t = (await app.page.locator('main, body').first().innerText()).replace(/\s+/g,' ');
    const s = t.includes('Cluster Connection Required')?'REQUIRED':t.includes('Connecting to cluster')?'CONNECTING':t.includes('Loading')?'LOADING':'OTHER';
    out.push(s);
    await app.page.waitForTimeout(1000);
  }
  console.log(r, out.join(','));
  await app.done();
}
