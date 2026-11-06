import { open } from './app.mjs';
const app = await open('tasks?project=all&view=board'); const p = app.page;
await p.setViewportSize({width:1024,height:600}); await p.waitForTimeout(700);
const root = p.locator('div.overflow-x-auto').filter({has:p.locator('[class*=overflow-y-auto]')}).first();
const box = await root.boundingBox(); console.log('box',JSON.stringify(box));
// drag scrollbar thumb: bar at bottom 10px; thumb starts at left
const y = box.y+box.height-5;
await p.mouse.move(box.x+40,y); await p.mouse.down(); await p.mouse.move(box.x+box.width-5,y,{steps:10}); await p.mouse.up();
console.log('scrollLeft', await root.evaluate(e=>[e.scrollLeft,e.scrollWidth-e.clientWidth]));
console.log('docScrollTop', await p.evaluate(()=>document.scrollingElement.scrollTop));
await app.shot('QA-076-after-dragged-1024x600');
// column vertical scroll: third column
const col = root.locator('> *').nth(2); const sc = col.locator('[class*=overflow-y-auto]').first();
const pos = async()=>({hdr:Math.round((await col.locator('> *').first().boundingBox()).y), top: await sc.evaluate(e=>e.scrollTop), ftr:Math.round((await col.locator('> *').last().boundingBox()).y)});
console.log('before',JSON.stringify(await pos()));
const sb = await sc.boundingBox(); await p.mouse.move(sb.x+100,sb.y+100); await p.mouse.wheel(0,600); await p.waitForTimeout(400);
console.log('after',JSON.stringify(await pos()), 'col1 scrollTop', await root.locator('> *').nth(0).locator('[class*=overflow-y-auto]').first().evaluate(e=>e.scrollTop));
await app.shot('QA-076-after-colscroll-1024x600');
console.log('footers', await root.locator('> *').evaluateAll(cs=>cs.map(c=>c.lastElementChild.innerText.slice(0,40))));
await app.done();
