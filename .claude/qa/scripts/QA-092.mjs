import { open } from './app.mjs';

const app = await open('cloud/workloads');
await app.page.waitForTimeout(1500);

const tiles = await app.page.evaluate(() => {
  const heading = [...document.querySelectorAll('h2,h3')].find((h) =>
    h.textContent.includes('Quick Actions')
  );
  const card = heading ? heading.closest('div')?.parentElement : null;
  const btns = card ? [...card.querySelectorAll('button')] : [];
  return btns.map((b) => b.textContent.trim());
});
console.log('Quick Action tiles:', JSON.stringify(tiles));
console.log('has Ingress tile:', tiles.some((t) => /ingress/i.test(t)));
console.log('has self-referential Workloads tile:', tiles.some((t) => /workloads/i.test(t)));

// Click "View Ingress" and confirm it navigates correctly.
const ingressTile = app.page.getByRole('button', { name: /view ingress/i });
if (await ingressTile.count()) {
  await ingressTile.first().click();
  await app.page.waitForTimeout(1000);
  console.log('after clicking View Ingress, url:', app.page.url());
}
await app.shot('QA-092-after-quick-actions');
await app.done();
