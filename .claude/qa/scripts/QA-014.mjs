// QA-014: a11y — "Enable Agent Platform" switch has no accessible name; <main> nested in <main> on all /settings pages.
import { open } from './app.mjs';
const app = await open('settings/ai');
const p = app.page;
console.log('switches without name:', await p.locator('[role=switch]').evaluateAll(els => els.filter(e => !(e.getAttribute('aria-label') || e.getAttribute('aria-labelledby') || e.labels?.length)).length));
console.log('nested <main>:', await p.evaluate(() => document.querySelectorAll('main main').length), '(expected 0)');
await app.shot('QA-014-before-ai-switch-unnamed');
await app.done();
