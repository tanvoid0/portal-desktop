// QA-015: at the minimum window (1024x600) the Settings page leaves a ~350px, ~380px-tall content pane.
import { open } from './app.mjs';
const app = await open('settings/ides', { width: 1024, height: 600 });
await app.page.setViewportSize({ width: 1024, height: 600 });
await app.page.waitForTimeout(500);
const box = await app.page.locator('main main').first().boundingBox();
console.log('content pane box', JSON.stringify(box));
await app.shot('QA-015-before-settings-min-window');
await app.page.setViewportSize({ width: 1280, height: 800 });
await app.done();
