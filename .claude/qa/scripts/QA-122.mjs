// Repro for QA-122: pending pairing request cards render the capability
// checklist's 3-column grid (CapabilityChecklist.svelte uses `sm:grid-cols-3`,
// a viewport-width breakpoint) inside a card far narrower than 640px, so at
// the normal app window width (>=640px) every label but the first in each
// row collides with the next row's switch.
import { open } from './app.mjs';

const app = await open('devices');
const { page } = app;
await page.waitForTimeout(500);
// Pending requests only render when there is at least one request waiting;
// the fixtures already on this machine's Devices page have two. If none are
// pending, pair a test device/browser tab first to reproduce.
await page.mouse.move(700, 400);
await page.mouse.wheel(0, 700);
await page.waitForTimeout(400);
await app.shot('QA-122-before-pending-capability-overlap');

const card = page.locator('text=Pending requests').locator('xpath=following-sibling::div[1]');
const labels = await card.locator('label').allTextContents();
console.log('capability labels in first pending card:', labels);
// Expect e.g. "agents.exec" and "dashboard" to read as whole words; when the
// bug is present they come through cut off ("agents.e", "dashboa") because
// the switch for the next column renders on top of the tail of the text.

await app.done();
