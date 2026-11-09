import { open } from './app.mjs';

const app = await open('cloud/deployments');
await app.page.waitForTimeout(1200);

// Open shortcuts help and read the Actions group.
await app.page.keyboard.press('?');
await app.page.waitForTimeout(500);
const helpText = await app.page.evaluate(() => document.body.innerText);
const restartLine = helpText.split('\n').find((l) => /restart/i.test(l));
console.log('Shortcuts help mentions Restart:', JSON.stringify(restartLine));
await app.shot('QA-093-shortcuts-help');
await app.page.keyboard.press('Escape');

// No deployments exist on this cluster, and creating one is out of scope (no cluster
// side effects) -- just confirm pressing "r" on the empty list doesn't error.
await app.page.keyboard.press('r');
await app.page.waitForTimeout(800);
console.log('after pressing r on empty list, url:', app.page.url());

await app.done();
