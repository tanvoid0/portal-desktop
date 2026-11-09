import { open } from './app.mjs';

async function checkRoute(route, label) {
  const app = await open(route);
  await app.page.waitForTimeout(3000);
  const state = await app.page.evaluate(() => {
    const sections = [...document.querySelectorAll('[aria-expanded]')].map(
      (el) => `${el.textContent.trim().slice(0, 25)}=${el.getAttribute('aria-expanded')}`
    );
    const pythonRow = [...document.querySelectorAll('a,button,div')].find((e) => e.textContent.trim() === 'Python');
    return {
      sections,
      pythonVisible: pythonRow ? pythonRow.getBoundingClientRect().height > 0 : false,
    };
  });
  console.log(`${label}:`, JSON.stringify(state));
  await app.shot(`QA-098-${label.replace(/[^a-z0-9]/gi, '-')}`);
  await app.done();
}

await checkRoute('sdk/python', 'direct-load-sdk-python');
await checkRoute('sdk', 'direct-load-sdk-overview');
