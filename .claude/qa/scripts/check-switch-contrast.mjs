import { open } from './app.mjs';
const app = await open('devices');
await app.page.waitForTimeout(500);
const info = await app.page.evaluate(() => {
  const sw = document.querySelector('[data-slot="switch"][data-state="unchecked"]');
  const card = sw?.closest('[data-slot="card"]') || sw?.closest('.rounded-lg, .rounded-xl');
  const swStyle = sw ? getComputedStyle(sw) : null;
  const cardStyle = card ? getComputedStyle(card) : null;
  return {
    switchBg: swStyle?.backgroundColor,
    switchBorder: swStyle?.borderColor,
    cardBg: cardStyle?.backgroundColor,
    bodyBg: getComputedStyle(document.body).backgroundColor,
  };
});
console.log(JSON.stringify(info, null, 2));
await app.done();
