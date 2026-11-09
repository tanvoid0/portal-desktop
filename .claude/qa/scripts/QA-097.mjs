// QA-097 verify: /sdk overview must load on a cold/direct load, not just via in-app nav.
// Hard-reload /sdk several times and confirm the counters + Quick Navigation render each time.
import { open } from './app.mjs';

for (let i = 0; i < 6; i++) {
  const app = await open('sdk');
  await app.page.waitForTimeout(3500); // give it generous time; the bug was an infinite hang
  const state = await app.page.evaluate(() => {
    const body = document.body.innerText;
    return {
      stillLoading: body.includes('Loading SDK data'),
      hasQuickNav: body.includes('Quick Navigation'),
      statsLine: (body.match(/Managers installed[\s\S]{0,80}/) || [null])[0],
    };
  });
  console.log(`attempt ${i}:`, JSON.stringify(state));
  if (i === 0 || state.stillLoading) await app.shot(`QA-097-attempt-${i}`);
  await app.done();
}
