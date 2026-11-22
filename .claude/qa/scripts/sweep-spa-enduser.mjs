import { open } from './app.mjs';
const app = await open('');
const p = app.page;
await p.waitForTimeout(1000);
console.log('dialog on home:', await p.getByText('Vault locked').count());
await p.getByRole('button',{name:'Not now'}).click().catch(()=>{});
for (const n of ['Tasks','Credentials','Devices','GitHub']) {
  await p.getByRole('link',{name:n,exact:true}).first().click().catch(e=>console.log('nolink',n));
  await p.waitForTimeout(1500);
  console.log(n,'dialog:', await p.getByText('Vault locked').count(), p.url());
  await p.getByRole('button',{name:'Not now'}).click({timeout:500}).catch(()=>{});
}
await app.shot('sweep-home-dark-1280');
await app.done();
