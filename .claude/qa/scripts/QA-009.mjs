// QA-009: after a reload, Settings "Save Changes" / "Export Settings" stay disabled (get(settings) is not reactive).
import { open } from './app.mjs';
const app = await open('');
const p = app.page;
const dis = async (n) => p.getByRole('button', { name: n }).isDisabled();
await p.getByRole('button', { name: 'Settings' }).last().click(); // in-app navigation
await p.waitForTimeout(1500);
console.log('in-app nav  :', new URL(p.url()).pathname, '| Save disabled:', await dis('Save Changes'), '| Export disabled:', await dis('Export Settings'));
await p.getByRole('button', { name: 'Refresh page' }).click(); // header reload button
await p.waitForTimeout(2000);
const s = await dis('Save Changes');
console.log('after reload:', new URL(p.url()).pathname, '| Save disabled:', s, '| Export disabled:', await dis('Export Settings'), s ? 'FAIL' : 'ok');
await p.getByLabel('Timezone').waitFor();
await app.shot('QA-009-before-save-disabled-after-reload');
await app.done();
