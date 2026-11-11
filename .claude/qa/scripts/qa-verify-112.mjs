// Independent re-verification of QA-112: "Accept & remember" must never
// appear on Agents → Run's approval banners (first-pending via CoderFeed,
// or any extra concurrent ones via ApprovalBanner), while coder's own
// existing usage elsewhere (/ai/coder) must be unaffected and still show it.
import { open } from './app.mjs';

const CWD = 'C:\\Users\\tanvo\\AppData\\Local\\Temp\\QA agent cwd with spaces';

// --- Part 1: Agents route, first-pending approval (CoderFeed slot) ---
{
  const app = await open('agents/run');
  const { page } = app;
  await page.getByPlaceholder('Pick a folder…').fill(CWD);
  await page
    .getByPlaceholder('What should the agent do?')
    .fill(
      'Use your Write tool to create a file named qa-verify112-a.txt with content "x", then tell me what you did.'
    );
  await page.getByRole('button', { name: 'Start run' }).click();

  let sawBanner = false;
  for (let i = 0; i < 60; i++) {
    await page.waitForTimeout(500);
    if ((await page.getByRole('button', { name: 'Accept once' }).count()) > 0) {
      sawBanner = true;
      break;
    }
  }
  const hasRemember = (await page.getByRole('button', { name: 'Accept & remember' }).count()) > 0;
  console.log('Agents route — saw approval banner:', sawBanner);
  console.log('Agents route — "Accept & remember" present:', hasRemember);
  await app.shot('qa-verify-112-agents-banner');
  // Clean up: accept so the run finishes, then end it.
  if (sawBanner) await page.getByRole('button', { name: 'Accept once' }).click();
  try {
    await page.getByRole('button', { name: 'End run' }).click({ timeout: 3000 });
  } catch {
    /* already ended */
  }
  await app.done();
}

// --- Part 2: coder's own route (/ai/coder) must be unaffected ---
{
  const app = await open('ai/coder');
  const { page } = app;
  // Just confirm the page loads and, if easily reachable, that the
  // ApprovalBanner's showRemember defaults true by reading CoderPanel's
  // CoderFeed call site is already confirmed via source grep; here we do a
  // light sanity check that the route itself renders without error.
  const bodyText = await page.locator('body').innerText();
  console.log('/ai/coder route loaded, body length:', bodyText.length);
  await app.shot('qa-verify-112-coder-route-sanity');
  await app.done();
}
