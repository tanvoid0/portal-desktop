// QA-021: Terminal view of a tab restored after reload shows ghost shell-path text on the prompt line. Needs an existing tab (run twice if clean).
import { open } from './app.mjs';
const app = await open('terminal');
const p = app.page;
await p.waitForTimeout(2000);
await p.getByLabel('New tab').click();
await p.waitForTimeout(2000);
await p.getByTitle('Terminal view (interactive apps)').locator('visible=true').click();
await p.waitForTimeout(3000);
await p.mouse.click(600, 300);
await p.keyboard.type('echo QA-xterm', { delay: 40 });
await p.keyboard.press('Enter');
await p.waitForTimeout(1500);
const t = await p.locator('.xterm-rows').locator('visible=true').first().innerText();
console.log('terminal text:', JSON.stringify(t.slice(0, 200)), '| ghost text:', /powershell\.exe/i.test(t));
await app.shot('QA-021-before-xterm-ghost-text');
await app.done();
