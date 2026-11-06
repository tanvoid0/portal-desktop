// Repro: /credentials renders a bare "500 Internal Error" when any credential exists (no secret values touched).
import { open } from './app.mjs';
const app = await open('credentials');
const text = (await app.page.locator('main').innerText()).trim();
console.log('main text:', JSON.stringify(text.slice(0, 80)), '(expected: page header + credential cards)');
await app.shot('QA-043-before-credentials-500');
await app.done();
