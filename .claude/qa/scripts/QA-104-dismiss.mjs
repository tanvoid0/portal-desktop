// QA-104: verify the vault setup dialog, once triggered from /credentials, can be
// dismissed via "Not now" / Escape / outside click without setting a real password,
// and that the app is usable afterward.
import { open } from './app.mjs';

const app = await open('credentials');
await app.page.waitForTimeout(1500);

async function dialogVisible() {
  return app.page.getByText('Set up your vault').count();
}

console.log('initial vault dialog present:', await dialogVisible());

// Try "Not now" button
const notNow = app.page.getByRole('button', { name: /not now/i });
const notNowCount = await notNow.count();
console.log('"Not now" button present:', notNowCount);
if (notNowCount > 0) {
  await notNow.first().click();
  await app.page.waitForTimeout(500);
  console.log('after clicking Not now, dialog present:', await dialogVisible());
}

// Re-trigger if needed, then try Escape
if ((await dialogVisible()) === 0) {
  // navigate to credentials again to re-trigger (setup likely gated on first action attempt)
  await app.page.reload();
  await app.page.waitForTimeout(1500);
  console.log('after reload, dialog present:', await dialogVisible());
}

if ((await dialogVisible()) > 0) {
  await app.page.keyboard.press('Escape');
  await app.page.waitForTimeout(500);
  console.log('after Escape, dialog present:', await dialogVisible());
}

// Confirm app usable: navigate to /projects and see it renders
await app.page.goto(new URL('/projects', app.page.url()).href);
await app.page.waitForTimeout(1000);
const heading = await app.page.getByText('Projects', { exact: true }).count();
console.log('after navigating to /projects post-dismiss, heading present:', heading);

await app.shot('QA-104-after-dismiss-usable');
await app.done();
