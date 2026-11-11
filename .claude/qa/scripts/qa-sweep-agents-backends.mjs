// Sweep: list available agent backends in the Run launcher's Select.
import { open } from './app.mjs';

const app = await open('agents/run');
await app.page.waitForTimeout(800);

await app.page.locator('#agent-backend').click();
await app.page.waitForTimeout(400);
const options = await app.page.locator('[role=option]').allInnerTexts();
console.log('backend options:', JSON.stringify(options));
await app.shot('sweep-agents-backend-options');

await app.page.keyboard.press('Escape');
await app.done();
