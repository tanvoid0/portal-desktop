// QA-013: Settings > Autonomy: backend errors ("Autonomy service not found") are swallowed; page shows defaults as if real.
import { open } from './app.mjs';
const app = await open('settings/autonomy');
console.log('page text has error/unavailable notice:', /unavailable|not found|failed|error/i.test(await app.page.locator('main').first().innerText()));
await app.shot('QA-013-before-autonomy-silent-error');
await app.done();
