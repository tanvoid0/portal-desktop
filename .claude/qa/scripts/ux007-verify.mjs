import { open } from './i2.mjs';
const app = await open('automation/blocks', { width: 1280, height: 800 });
const page = app.page;
await page.waitForTimeout(2500);
const m = async () => page.evaluate(() => {
  const cards = [...document.querySelectorAll('[data-slot="card"]')].filter(c => c.getBoundingClientRect().height > 40 && c.getBoundingClientRect().y > 150);
  const first = cards[0];
  return {
    chromeToFirstCard: first ? Math.round(first.getBoundingClientRect().top) : null,
    cardH: first ? Math.round(first.getBoundingClientRect().height) : null,
    cardW: first ? Math.round(first.getBoundingClientRect().width) : null,
    cards: cards.length,
    h1: [...document.querySelectorAll('h1')].map(h=>h.innerText.trim()),
    hscroll: document.documentElement.scrollWidth > window.innerWidth + 1,
    controlRows: [...document.querySelectorAll('button, input')].filter(e=>{const r=e.getBoundingClientRect(); return r.y>60 && r.y<300 && r.width>0;}).map(e=>Math.round(e.getBoundingClientRect().y)).filter((v,i,a)=>a.indexOf(v)===i).sort((a,b)=>a-b),
  };
});
console.log('1280x800:', JSON.stringify(await m()));
await app.shot('UX-007-after-blocks-1280');
await app.size(1024, 600);
await page.waitForTimeout(800);
console.log('1024x600:', JSON.stringify(await m()));
await app.shot('UX-007-after-blocks-1024x600');
await app.size(1280, 800);
console.log('buttons:', await page.evaluate(()=>[...document.querySelectorAll('main button')].map(b=>b.innerText.trim().replace(/\n/g,' / ')).filter(Boolean).slice(0,22)));
await app.done();
