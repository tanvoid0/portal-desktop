// Smoke: shared shell on Portal + Claude Code tabs after #1212 merge.
import { open } from './app.mjs';
const app = await open('ai/coder');
const { page } = app;
const txt = async () => (await page.evaluate(() => document.querySelector('main')?.innerText ?? document.body.innerText)).replace(/\n+/g, ' | ');
console.log('PORTAL:', (await txt()).slice(0, 500));
await app.shot(`ux-coder-${process.env.TAG ?? 'after'}-portal`);
await page.getByText('Claude Code', { exact: true }).first().click();
await page.waitForTimeout(2000);
console.log('CLAUDE EMPTY:', (await txt()).slice(0, 400));
await app.shot(`ux-coder-${process.env.TAG ?? 'after'}-claude-empty`);
await page.getByText(process.argv[2]).first().click();
await page.waitForTimeout(3500);
const feed = await page.evaluate(() => document.querySelector('[data-slot="scroll-area-viewport"]')?.innerText.length ?? -1);
console.log('CLAUDE SESSION feed chars:', feed);
console.log('CLAUDE SESSION:', (await txt()).slice(0, 700));
await app.shot(`ux-coder-${process.env.TAG ?? 'after'}-claude-session`);
await app.done();
